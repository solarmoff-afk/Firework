// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use proc_macro2::TokenTree;
use syn::{Stmt, Expr, Error};
use syn::{UnOp, BinOp};
use quote::ToTokens;
use std::collections::HashSet;

use super::widgets::is_widget_macro;
use super::codegen::actions::{FireworkStatement, FireworkAction, WidgetType};
use super::{
    compile_error_spanned, is_mutable_method,
    SPARK_USAGE_ERROR, SPARK_SHADOWING_ERROR, SPARK_TYPE_ERROR, SPARK_UNIQUE_NAME_ERROR,
};

/// Структура которая собирает метаинформацию о каждой функции в ui блоке
#[derive(Clone)]
pub struct ItemMetadata {
    sparks: HashSet<String>,
    variables: HashSet<String>,
}

impl ItemMetadata {
    pub fn default() -> Self {
        Self {
            sparks: HashSet::new(),
            variables: HashSet::new(),
        }
    }

    pub fn clear(&mut self) {
        self.sparks.clear();
        self.variables.clear();
    }
}

#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    pub name: String,
    pub ty: Option<String>,
    pub is_mut: bool,
}

pub struct CompilerContext {
    pub depth: usize,
    pub active_targets: Vec<VariableDeclaration>,
    pub is_mutation: bool,
    pub metadata: ItemMetadata,

    // Определяет явлемся ли мы правой частью присваивания. Это нужно чтобы понять
    // относится ли вызов макроса (например spark!) к переменной или он вызван не там
    // где нужно
    pub is_right_side: bool,
    
    // Находимся ли мы в присваивании
    pub is_assign: bool,

    // Вектор ошибок компиляции чтобы накопить их
    pub compile_errors: Vec<Error>,

    // Последняя переменная найденная в local ветке
    pub variable_name: String,

    pub variable_type: String,

    // Statement это блок кода от начала до ; или фигурных скобок. Нужно точно знать
    // на каком statement мы сейчас. На старте это 0, поэтому итерацию нужно начать
    // с нуля
    pub statement_index: usize,

    pub statements: Vec<FireworkStatement>,
    pub last_statement: FireworkStatement,

    // Флаг который включается когда спарк может быть изменён, но чтобы узнать точную
    // переменную нужно пропарсить expr и залезть в path, этот флаг нужен чтобы ветка
    // path поняла что сейчас ожидается спарк на изменение и запустила проверку
    pub spark_mut_maybe: bool,
}

impl CompilerContext {
    fn indent(&self) -> String {
        "  ".repeat(self.depth)
    }

    fn log(&self, label: &str, details: &str) {
        let targets: Vec<String> = self.active_targets
            .iter()
            .map(|v| {
                if let Some(ty) = &v.ty {
                    format!("{}: {}", v.name, ty)
                } else {
                    v.name.clone()
                }
            })
            .collect();
            
        let targets_str = if targets.is_empty() {
            "NONE".to_string()
        } else {
            targets.join(", ")
        };
        
        // println!("{}[{}] Target: [{}] | Mutation: {} | Details: {}", self.indent(), label, targets_str, self.is_mutation, details);
    }
}

pub fn prepare_tokens(tokens: Vec<TokenTree>) -> (proc_macro2::TokenStream, Option<proc_macro2::TokenStream>) {
    let mut context = CompilerContext {
        depth: 0,
        active_targets: Vec::new(),
        is_mutation: false,
        metadata: ItemMetadata::default(),
        is_right_side: false,
        is_assign: false,
        compile_errors: Vec::new(),
        variable_name: String::from(""),
        variable_type: String::from(""),
        statement_index: 0,
        statements: Vec::new(),

        last_statement: FireworkStatement {
            action: FireworkAction::DefaultCode,
            index: 0,
        },

        spark_mut_maybe: false,
    };

    let token_stream: proc_macro2::TokenStream = tokens.clone().into_iter().collect();

    let parser = |input: syn::parse::ParseStream| -> syn::Result<Vec<syn::Item>> {
        let mut items = Vec::new();
        
        while !input.is_empty() {
            items.push(input.parse::<syn::Item>()?);
        }
        
        Ok(items)
    };

    let items = match syn::parse::Parser::parse2(parser, token_stream) {
        Ok(items_vec) => items_vec,
        Err(message) => {
            return (proc_macro2::TokenStream::new(), Some(message.to_compile_error()));
        },
    }; 

    let mut output = proc_macro2::TokenStream::new();
    let mut error_tokens = None;
    
    for item in items {
        output.extend(parse_items(item, &mut context));
    }

    if !context.compile_errors.is_empty() {
        let mut final_error = context.compile_errors.remove(0);
        
        for error in context.compile_errors {
            final_error.combine(error);
        }

        error_tokens = Some(final_error.to_compile_error());
    }

    println!("{:#?}", context.statements);
    
    (output, error_tokens)
}

/// Парсит statement, это конкретная команда. Можно упростить и сказать что это
/// строки, но это не совсем так
fn parse_stmts(statements: Vec<Stmt>, context: &mut CompilerContext) {
    for statement in statements {
        println!("STATEMENT:");
        println!("{:#?}", statement);
        
        context.last_statement.action = FireworkAction::DefaultCode;

        match statement {
            Stmt::Local(local) => {
                parse_local(local, context);
            },
            
            Stmt::Item(item) => {
                parse_items(item, context);
            },

            Stmt::Expr(expr, _semi) => {
                parse_expr(expr, context);
            },

            Stmt::Macro(mac) => {
                if is_widget_macro(&mac.mac.path) {
                    let inner_tokens = &mac.mac.tokens;

                    let _parser = |input: syn::parse::ParseStream| -> syn::Result<Vec<Stmt>> {
                        let mut stmts = Vec::new();

                        while !input.is_empty() {
                            stmts.push(input.parse::<syn::Stmt>()?);
                        }

                        Ok(stmts)
                    };

                    let inner_stmts: Vec<Stmt> = syn::parse2(quote::quote! {
                        {
                            #inner_tokens
                        }
                    }).and_then(|block: syn::Block| Ok(block.stmts))
                        .expect("Syntax error in macro");
                    
                    parse_stmts(inner_stmts, context);
                } else {
                    if let Some(last_segment) = mac.mac.path.segments.last() {
                        let _macro_name = &last_segment.ident;
                        
                        // Это обычный макрос, нужен инлайн
                    }
                }
            },
        };

        println!("Statement index: {}", context.statement_index);
        context.statement_index += 1;
        context.last_statement.index += 1;

        context.statements.push(context.last_statement.clone());
    }
}

/// Парсит создание переменной
fn parse_local(local: syn::Local, context: &mut CompilerContext) {
    parse_pat(local.pat, None, context);

    if let Some(init) = local.init {
        context.is_right_side = true;
        context.depth += 1;
            parse_expr(*init.expr, context);
            
            // Нужно добавлять переменную только после обработки макроса spark чтобы
            // избежать механизма который бросает FE004, обработка макроса spark как раз
            // находится в одной из веток parse_expr, поэтому делаем insert только после
            // вызова parse_expr
            context.metadata.variables.insert(context.variable_name.clone());
        context.depth -= 1;
        context.is_right_side = false;
    }
}

/// Парсит паттерн, это может быть a, (a, b), [a, b, c] и так далее
fn parse_pat(pat: syn::Pat, current_type: Option<String>, context: &mut CompilerContext) {
    match pat {
        syn::Pat::Ident(ident) => {
            let variable = VariableDeclaration {
                name: ident.ident.to_string(),
                ty: current_type.clone(),
                is_mut: ident.mutability.is_some(),
            };

            let variable_type = current_type.unwrap_or("NO TYPE".to_string()); 

            println!(
                "{}Let: is_mut: {}, name: {}, type: {}",
                context.indent(), variable.is_mut, variable.name,
                variable_type,
            );

            context.variable_name = ident.ident.to_string();
            context.variable_type = variable_type;

            if context.metadata.sparks.contains(&context.variable_name) {
                context.compile_errors.push(compile_error_spanned(
                    &ident,
                    SPARK_SHADOWING_ERROR,
                ));
            }

            context.active_targets.push(variable); 
        },

        syn::Pat::Type(pat_type) => {
            let type_str = pat_type.ty.to_token_stream().to_string();
            
            // Если это кортеж с типом
            if let syn::Type::Tuple(type_tuple) = &*pat_type.ty {
                // Для каждого элемента кортежа свой тип
                if let syn::Pat::Tuple(pat_tuple) = &*pat_type.pat {
                    for (i, elem_pat) in pat_tuple.elems.iter().enumerate() {
                        let elem_type = type_tuple.elems.get(i).map(|t| t.to_token_stream().to_string());
                        parse_pat(elem_pat.clone(), elem_type, context);
                    }
                
                    return;
                }
            }
    
            parse_pat(*pat_type.pat, Some(type_str), context);
        },

        syn::Pat::Tuple(pat_tuple) => {
            for element in pat_tuple.elems.iter() {
                parse_pat(element.clone(), None, context);
            }
        },

        syn::Pat::Struct(pat_struct) => {
            for field in pat_struct.fields.iter() {
                parse_pat(*field.pat.clone(), None, context);
            }
        },

        syn::Pat::Slice(pat_slice) => {
            for (_index, element) in pat_slice.elems.iter().enumerate() {
                parse_pat(element.clone(), None, context);
            }
        },

        syn::Pat::Or(pat_or) => {
            for (_index, case) in pat_or.cases.iter().enumerate() {
                parse_pat(case.clone(), None, context);
            }
        },

        _ => {},
    };
}

/// Парсит выражение. Выражение это почти всё что существует. Break, Continue,
/// 2 + 2, x += 1, услоовие в if и так далее
pub fn parse_expr(expression: syn::Expr, context: &mut CompilerContext) {
    match expression {
        // Массив ( [a, b, c, d] )
        Expr::Array(expression_array) => {
            context.log("ARRAY", "Entering array elements");
            context.depth += 1;
            
            for element in expression_array.elems {
                parse_expr(element, context);
            }
            
            context.depth -= 1;
        },

        // Присваивание (a = compute())
        Expr::Assign(expression_assign) => {
            let left_name = expression_assign.left.to_token_stream().to_string();
            context.log("ASSIGN_START", &format!("Target: {}", left_name));

            context.is_assign = true;

            // Попытка изменить значение spark
            if context.metadata.sparks.contains(&left_name) {
                context.last_statement.action = FireworkAction::SparkUpdate(left_name.clone());
            }

            let previous_targets = context.active_targets.clone();
            let previous_mutation_state = context.is_mutation;

            context.active_targets = vec![VariableDeclaration {
                name: left_name,
                ty: None,
                is_mut: true,
            }];
            context.is_mutation = true;

            context.depth += 1;
                parse_expr(*expression_assign.left, context);
                parse_expr(*expression_assign.right, context);
            context.depth -= 1;

            context.active_targets = previous_targets;
            context.is_mutation = previous_mutation_state;
            context.log("ASSIGN_END", "");

            context.is_assign = false;
        },

        // Асинхронность ( async { ... } )
        Expr::Async(expression_async) => {
            context.log("ASYNC_BLOCK", "Entering async block");
            context.depth += 1;
            
            parse_stmts(expression_async.block.stmts, context);
            context.depth -= 1;
        },

        // fut.await
        Expr::Await(expression_await) => {
            context.log("AWAIT", "Awaiting expression");
            parse_expr(*expression_await.base, context);
        },

        // Бинарные операции (a + b, a += b)
        Expr::Binary(expression_binary) => {
            let operator = expression_binary.op.to_token_stream().to_string();
            
            let op_type = match expression_binary.op {
                BinOp::Add(_) => "ADD",
                BinOp::Sub(_) => "SUB",
                BinOp::Mul(_) => "MUL",
                BinOp::Div(_) => "DIV",
                BinOp::Rem(_) => "REM",
                BinOp::And(_) => "AND",
                BinOp::Or(_) => "OR",
                BinOp::BitXor(_) => "BIT_XOR",
                BinOp::BitAnd(_) => "BIT_AND",
                BinOp::BitOr(_) => "BIT_OR",
                BinOp::Shl(_) => "SHL",
                BinOp::Shr(_) => "SHR",
                BinOp::Eq(_) => "EQ",
                BinOp::Lt(_) => "LT",
                BinOp::Le(_) => "LE",
                BinOp::Ne(_) => "NE",
                BinOp::Ge(_) => "GE",
                BinOp::Gt(_) => "GT",

                // Заглушка
                _ => "BINARY_OP",
            };

            let assign_op = match expression_binary.op {
                BinOp::AddAssign(_) => true,
                BinOp::SubAssign(_) => true,
                BinOp::MulAssign(_) => true,
                BinOp::DivAssign(_) => true,
                BinOp::RemAssign(_) => true,
                BinOp::BitXorAssign(_) => true,
                BinOp::BitAndAssign(_) => true,
                BinOp::BitOrAssign(_) => true,
                BinOp::ShlAssign(_) => true,
                BinOp::ShrAssign(_) => true,
                _ => false,
            };

            if assign_op {
                // Ветка path поймёт что флаг включён и пометить этот стейтемент
                // как мутацию спарка
                context.spark_mut_maybe = true;
            }
            
            context.log(op_type, &format!("Operator: {}", operator));
            
            context.depth += 1;
                parse_expr(*expression_binary.left, context);

                // Для правой части проверка спарка не нужна
                context.spark_mut_maybe = false; 
                parse_expr(*expression_binary.right, context);
            context.depth -= 1;
        },

        // Блок ( { ... } )
        Expr::Block(expression_block) => {
            context.log("BLOCK_EXPR", "Entering block");
            
            // Сохранение метеданных (спарки и переменные) для области видимости
            // до блока. Спарки и сигналы будут в метаданных, но после того как
            // блок завершён то нужно сбросить метаданные до статуса в котором
            // они были до входа в новую область видимости. Если визуализировать:
            // fn ...(...) {
            //  // Тут одна область видимости
            //  let mut spark1: u32 = spark!(0);
            //
            //  // Тут другая область видимости
            //  {
            //      let mut spark2: u32 = spark!(0);
            //
            //      // Тут можно работать и с spark1 и с spark2
            //  }
            //
            //  // Тут срабатывает строка context.metadata = global_metadata;
            //  // Теперь доступ есть только к spark1 так как он находится в
            //  // в этой области видимости
            //  spark1 += 1;
            // }
            let global_metadata = context.metadata.clone();

            context.depth += 1;
                parse_stmts(expression_block.block.stmts, context);
            context.depth -= 1;

            context.metadata = global_metadata;
        },

        // Break (выход из цикла)
        Expr::Break(expression_break) => {
            context.log("BREAK", "");
            
            if let Some(break_expression) = expression_break.expr {
                parse_expr(*break_expression, context);
            }
        },

        // Вызов функции (invoke(a, b))
        Expr::Call(expression_call) => {
            let func_name = expression_call.func.to_token_stream().to_string();
            context.log("CALL", &format!("Function: {}", func_name));
            
            context.depth += 1;
                for argument in expression_call.args {
                    parse_expr(argument, context);
                }
            context.depth -= 1;
        },

        // Каст (foo as f64)
        Expr::Cast(expression_cast) => {
            let type_name = expression_cast.ty.to_token_stream().to_string();
            
            context.log("CAST", &format!("To type: {}", type_name));
            parse_expr(*expression_cast.expr, context);
        },
        
        // Замыкание ( |a, b| a + b )
        Expr::Closure(expression_closure) => {
            context.log("CLOSURE", "Entering closure body");
            
            context.depth += 1;
                parse_expr(*expression_closure.body, context);
            context.depth -= 1;
        },

        // Константа ( const { ... } )
        Expr::Const(expression_const) => {
            context.log("CONST_BLOCK", "Entering const block");
            context.depth += 1;
            parse_stmts(expression_const.block.stmts, context);
            context.depth -= 1;
        },

        // Скипает шаг цикла
        Expr::Continue(_expression_continue) => {
            context.log("CONTINUE", "");
        },

        // Обращение по полю экземлпяра структуры (obj.k или для структур у которых поля
        // без имени obj.0)
        Expr::Field(expression_field) => {
            let member = expression_field.member.to_token_stream().to_string();
            context.log("FIELD_ACCESS", &format!("Member: .{}", member));
           
            // Base внутри это путь который ведёт к левой части работы с полем
            // (То есть если стейтемент a.push(1) то a это база)
            context.spark_mut_maybe = context.is_assign; // Copy чтобы избежать условия
                parse_expr(*expression_field.base, context);
            context.spark_mut_maybe = false;
        },

        // for i in collection { ... }
        Expr::ForLoop(expression_for_loop) => {
            let pattern = expression_for_loop.pat.to_token_stream().to_string();
            context.log("FOR_LOOP", &format!("Pattern: {}", pattern));
            
            context.depth += 1;
                parse_expr(*expression_for_loop.expr, context);
                parse_stmts(expression_for_loop.body.stmts, context);
            context.depth -= 1;
        },

        // Просто контейнер для выражения
        Expr::Group(expression_group) => {
            parse_expr(*expression_group.expr, context);
        },

        // Условие ( if expr { ... } else { ... } )
        Expr::If(expression_if) => {
            context.log("IF_START", "Condition:");
            
            context.depth += 1;
                parse_expr(*expression_if.cond, context);
                context.log("IF_THEN", "Then branch:");
            
                parse_stmts(expression_if.then_branch.stmts, context);
               
                // Если есть else
                if let Some((_else_token, else_expression)) = expression_if.else_branch {
                    context.log("IF_ELSE", "Else branch:");
                    parse_expr(*else_expression, context);
                }
            context.depth -= 1;
        },

        // Обращение по индексу (vector[2])
        Expr::Index(expression_index) => {
            context.log("INDEX_ACCESS", "");
            parse_expr(*expression_index.expr, context);
            parse_expr(*expression_index.index, context);
        },

        // _
        Expr::Infer(_expression_infer) => {
            context.log("INFER", "Underscore _");
        },

        // let внутри выражения (например внутри условия или цикла, let Some(x) = opt)
        Expr::Let(expression_let) => {
            let pattern = expression_let.pat.to_token_stream().to_string();
            context.log("LET_EXPR", &format!("Pattern: {}", pattern));
            
            parse_expr(*expression_let.expr, context);
        },

        // Литерал, это 1, "foo" и так далее
        Expr::Lit(expression_literal) => {
            context.log("LITERAL", &expression_literal.to_token_stream().to_string());
        },

        // Бесконечный цикл ( loop { ... } )
        Expr::Loop(expression_loop) => {
            context.log("LOOP", "Entering infinite loop");
            
            context.depth += 1;
                parse_stmts(expression_loop.body.stmts, context);
            context.depth -= 1;
        },

        // Вызов макроса в выражении, например format!("{}", q)
        Expr::Macro(expression_macro) => {
            let macro_name = expression_macro.mac.path.to_token_stream().to_string();
            context.log("MACRO", &format!("{}!", macro_name));
            
            // Хардкод для теста
            if macro_name == "spark" {
                // [FE001]
                // Если спарк создаётся не как правая часть создания переменной то ошибка 
                // омпиляции, так нельзя
                if !context.is_right_side {
                    context.compile_errors.push(compile_error_spanned(
                        &expression_macro.mac,
                        SPARK_USAGE_ERROR,
                    ));
                } else {
                    // Мы точно знаем что имя в контексте это имя спарка, так как
                    // мы точно находимся в правой части локальной ветки. Мы можем
                    // попасть сюда только если мы в объявлении переменной в правой
                    // части которой используется spark!, а в local ветке мы записываем
                    // имя в context.variable_name
                    let variable_name = &context.variable_name;

                    // Если спарку не задан тип то эта ошибка, так как спарк должен быть
                    // определён в структуре на следующих этапах компиляции
                    if context.variable_type == "NO TYPE" {
                        context.compile_errors.push(compile_error_spanned(
                            &expression_macro.mac,
                            SPARK_TYPE_ERROR,
                        ));
                    }
                    
                    context.metadata.sparks.insert(variable_name.to_string());
                    context.last_statement.action = FireworkAction::InitialSpark(
                        context.variable_name.clone(), context.variable_type.clone(),
                    );
                }

                // [FE004]
                // Переменной с таким именем не должно существовать
                if context.metadata.variables.contains(&context.variable_name) {
                    context.compile_errors.push(compile_error_spanned(
                        &expression_macro.mac,
                        SPARK_UNIQUE_NAME_ERROR,
                    ));
                } 

                context.log("SPARK_INIT", "Parsing spark content");
                
                let inner_expression: Expr = syn::parse2(expression_macro.mac.tokens)
                    .expect("Failed to parse tokens inside spark");
                
                context.depth += 1;
                    parse_expr(inner_expression, context);
                context.depth -= 1;
            }
        },

        // Матч ( match n { Some(n) => {}, None => {} } )
        Expr::Match(expression_match) => {
            context.log("MATCH", "Matching expression:");
            
            context.depth += 1;
                parse_expr(*expression_match.expr, context);
                
                for arm in expression_match.arms {
                    let arm_pat = arm.pat.to_token_stream().to_string();
                    context.log("MATCH_ARM", &format!("Arm pattern: {}", arm_pat));
                
                    if let Some(guard) = arm.guard {
                        context.log("MATCH_GUARD", "Guard condition:");
                        parse_expr(*guard.1, context);
                    }
                    
                    parse_expr(*arm.body, context);
                }
            context.depth -= 1;
        },

        // Вызов метода x.foo::<T>(a, b)
        // TODO: Сейчас вызов любого метода спарка считается как мутация самого
        // спарка, нужно добавить логику для того чтобы отличить мутабельные
        // методы и имутабельные для стандартных типов раст
        Expr::MethodCall(expression_method_call) => {
            let method = expression_method_call.method.to_string();
            context.log("METHOD_CALL", &format!("Method: .{}()", method));
            
            context.depth += 1;
                context.spark_mut_maybe = true;
                    parse_expr(*expression_method_call.receiver, context);
                context.spark_mut_maybe = false;

                for argument in expression_method_call.args {
                    parse_expr(argument, context);
                }
            context.depth -= 1;
        },
        
        // (a + b)
        Expr::Paren(expression_paren) => {
            parse_expr(*expression_paren.expr, context);
        },

        // Путь
        Expr::Path(expression_path) => {
            let path = expression_path.to_token_stream().to_string();
            context.log("PATH_USAGE", &format!("Variable: {}", path));

            // Если это выражение часть мутации спарка то фиксируем это в контексте 
            if context.spark_mut_maybe {
                if context.metadata.sparks.contains(&path) {
                    context.last_statement.action = FireworkAction::SparkUpdate(path);
                }
                
                context.spark_mut_maybe = false;
            }
        },

        // 1..2, 1.., ..2, 1..=2, ..=2
        Expr::Range(expression_range) => {
            context.log("RANGE", "");
            
            if let Some(start) = expression_range.start {
                parse_expr(*start, context);
            }
            
            if let Some(end) = expression_range.end {
                parse_expr(*end, context);
            }
        },

        // Использование ссылки на переменной, &a или &mut a
        Expr::Reference(expression_reference) => {
            context.log("REFERENCE", "Taking reference");
            parse_expr(*expression_reference.expr, context);
        },

        // Массив который состоит из одного элемента который повторяется
        // [0u8; N]
        Expr::Repeat(expression_repeat) => {
            context.log("ARRAY_REPEAT", "");
            parse_expr(*expression_repeat.expr, context);
            parse_expr(*expression_repeat.len, context);
        },

        // return
        Expr::Return(expression_return) => {
            context.log("RETURN", "");
            if let Some(return_expression) = expression_return.expr {
                parse_expr(*return_expression, context);
            }
        },

        // Point { x: 1, y: 1 }
        Expr::Struct(expression_struct) => {
            let struct_name = expression_struct.path.to_token_stream().to_string();
            context.log("STRUCT_INIT", &format!("Struct: {}", struct_name));
            
            context.depth += 1;
                for field in expression_struct.fields {
                    let field_name = field.member.to_token_stream().to_string();
                    context.log("FIELD_INIT", &format!("Field: {}", field_name));
                    
                    parse_expr(field.expr, context);
                }
                
                if let Some(rest_expression) = expression_struct.rest {
                    context.log("STRUCT_REST", "");
                    parse_expr(*rest_expression, context);
                }
            context.depth -= 1;
        },

        // expr? (Оператор ?)
        Expr::Try(expression_try) => {
            context.log("TRY_OP", "Using ? operator");
            parse_expr(*expression_try.expr, context);
        },

        // try { ... }
        Expr::TryBlock(expression_try_block) => {
            context.log("TRY_BLOCK", "Entering try block");
            
            context.depth += 1;
                parse_stmts(expression_try_block.block.stmts, context);
            context.depth -= 1;
        },

        // Кортеж (a, b, c, d)
        Expr::Tuple(expression_tuple) => {
            context.log("TUPLE", "Entering tuple elements");
            
            context.depth += 1;
                for element in expression_tuple.elems {
                    parse_expr(element, context);
                }
            context.depth -= 1;
        },

        // Унарные операторы, это минус, НЕ, дереф (*x, !x, -x)
        Expr::Unary(expression_unary) => {
            let operator = expression_unary.op.to_token_stream().to_string();
            
            let op_type = match expression_unary.op {
                UnOp::Deref(_) => "DEREF",
                UnOp::Not(_) => "NOT",
                UnOp::Neg(_) => "NEG",
                _ => "OTHER_UNARY_OP",
            };
            
            context.log(op_type, &format!("Operator: {}", operator));
            parse_expr(*expression_unary.expr, context);
        },

        // unsafe { ... } блок
        Expr::Unsafe(expression_unsafe) => {
            context.log("UNSAFE_BLOCK", "Entering unsafe block");
            
            context.depth += 1;
                parse_stmts(expression_unsafe.block.stmts, context);
            context.depth -= 1;
        },

        // Цикл while expr { ... }
        Expr::While(expression_while) => {
            context.log("WHILE_LOOP", "Condition:");
            
            context.depth += 1;
                parse_expr(*expression_while.cond, context);
                context.log("WHILE_BODY", "Body:");

                parse_stmts(expression_while.body.stmts, context);
            context.depth -= 1;
        },

        // Night фича компилятора раст для коротин
        Expr::Yield(expression_yield) => {
            context.log("YIELD", "");

            if let Some(yield_expression) = expression_yield.expr {
                parse_expr(*yield_expression, context);
            }
        },

        // Выражение не кушает syn
        Expr::Verbatim(token_stream) => {
            context.log("VERBATIM", &token_stream.to_string());
        },

        _ => {
            context.log("UNKNOWN", "");
        }
    }
}

/// Парсит предметы. Предмет это верхушка пищевой цепи в Rust, это модули, функции,
/// трейты, структуры и так далее
fn parse_items(item: syn::Item, context: &mut CompilerContext) -> proc_macro2::TokenStream {
    let mut output = proc_macro2::TokenStream::new();
    
    match &item {
        // TEMP: Заглушка чтобы компилятор не жаловался пока макрос в разработке
        syn::Item::Fn(item_fn) => {
            let fn_name = item_fn.sig.ident.to_string();
            context.log("fn_found", &format!("function: {}", fn_name));

            // Если context.depth это ноль то мы находимся в корне ui! блока, а значит
            // заходим в функцию экрана. Архитектура фреймворка разрешает делать несколько
            // экранов (функций) в одном ui! блоке поэтому для каждого экрана должны
            // быть чистые метаданные, поэтому очищаем их
            context.metadata.clear();
            
            context.depth += 1;
                parse_stmts(item_fn.block.stmts.clone(), context);
            context.depth -= 1;
            
            context.log("FN_STUB", &format!("Generating stub for: {}", fn_name));
            
            let mut fn_stub = item_fn.clone();
            fn_stub.block = syn::parse2(quote::quote! {
                {}
            }).expect("Failed to parse item");

            output.extend(quote::quote! {
                #fn_stub
            });
        },
        
        syn::Item::Struct(item_struct) => {
            let struct_name = item_struct.ident.to_string();
            context.log("STRUCT_PASSTHROUGH", &format!("Struct: {}", struct_name));

            output.extend(quote::quote! {
                #item
            });
        },
        
        syn::Item::Enum(item_enum) => {
            let enum_name = item_enum.ident.to_string();
            context.log("ENUM_PASSTHROUGH", &format!("Enum: {}", enum_name));
            
            output.extend(quote::quote! {
                #item
            });
        },
        
        syn::Item::Type(item_type) => {
            let type_name = item_type.ident.to_string();
            context.log("TYPE_PASSTHROUGH", &format!("Type alias: {}", type_name));
            
            output.extend(quote::quote! {
                #item
            });
        },
        
        syn::Item::Const(item_const) => {
            let const_name = item_const.ident.to_string();
            context.log("CONST_PASSTHROUGH", &format!("Const: {}", const_name));
            
            output.extend(quote::quote! {
                #item
            });
        },
        
        syn::Item::Static(item_static) => {
            let static_name = item_static.ident.to_string();
            context.log("STATIC_PASSTHROUGH", &format!("Static: {}", static_name));
            
            output.extend(quote::quote! {
                #item
            });
        },
        
        syn::Item::Trait(item_trait) => {
            let trait_name = item_trait.ident.to_string();
            context.log("TRAIT_PASSTHROUGH", &format!("Trait: {}", trait_name));
            output.extend(quote::quote! {
                #item
            });
        },
        
        syn::Item::Impl(_item_impl) => {
            context.log("IMPL_PASSTHROUGH", "Implementation block");
            output.extend(quote::quote! {
                #item
            });
        },
        
        syn::Item::Mod(item_mod) => {
            let mod_name = item_mod.ident.to_string();
            context.log("MOD_PASSTHROUGH", &format!("Module: {} (keeping original)", mod_name));
            
            output.extend(quote::quote! {
                #item_mod
            });
        },

        syn::Item::Use(_item_use) => {
            context.log("USE_PASSTHROUGH", "Use statement");
            
            output.extend(quote::quote! {
                #item
            });
        },
        
        syn::Item::ExternCrate(item_extern) => {
            context.log("EXTERN_CRATE_PASSTHROUGH", &format!("Extern crate: {}", item_extern.ident));
            output.extend(quote::quote! {
                #item
            });
        },
        
        syn::Item::ForeignMod(_item_foreign) => {
            context.log("FOREIGN_MOD_PASSTHROUGH", "Foreign module");
            
            output.extend(quote::quote! {
                #item
            });
        },
        
        syn::Item::Macro(item_macro) => {
            if let Some(last_segment) = item_macro.mac.path.segments.last() {
                let macro_name = &last_segment.ident;
                context.log("MACRO_ITEM_PASSTHROUGH", &format!("Macro: {}!", macro_name));
            }

            output.extend(quote::quote! {
                #item
            });
        },
        
        syn::Item::Verbatim(tokens) => {
            context.log("VERBATIM_PASSTHROUGH", "Raw tokens");
            output.extend(tokens.clone());
        },
        
        _ => {
            context.log("UNKNOWN_ITEM", "Unknown item type");
            
            output.extend(quote::quote! {
                #item
            });
        }
    };
    
    output
}
