// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

mod spark;
mod widget;

use proc_macro2::{TokenTree, TokenStream, Span};
use syn::*;
use syn::spanned::Spanned;
use syn::visit::Visit;
use std::collections::{HashMap, HashSet};
use quote::ToTokens;

use widget::{is_widget, is_layout, map_skin, WidgetArgs};
use spark::{SparkValidator, SparkFinder, get_root_variable_name};

use crate::compiler::utils::is_mutable_method;
use crate::compiler::codegen::actions::{FireworkIR, FireworkStatement, FireworkAction};
use crate::{
    compile_error_spanned, SPARK_MULTIPLE_ERROR, SPARK_SHADOWING_ERROR,
    SPARK_UNIQUE_NAME_ERROR, SPARK_TYPE_ERROR, WIDGET_PARSE_ERROR, LAYOUT_PARSE_ERROR,
    LAYOUT_MULTIPLE_ERROR,
};

/// Нельзя хранить String поэтому используется &str, при использовании нужно использовать
/// String::from, но это позволяет не тянуть lazy_static или другой крейт
pub const NO_TYPE: &str = "NO TYPE";

/// Структура для декларации переменной в структуре области видимости
#[derive(Debug, Clone)]
pub struct Variable {
    // Тип переменной строкой, если не указан то он останется NO_TYPE
    pub variable_type: String,

    // Явлется ли эта переменная реактивной (спарком). Это определяется по налиию
    // макроса spark!() в выражении, но будет ошибка если имя спарка не будет
    // уникальным, если:
    //
    // 1 кейс: Другая переменная затенит спарк (shadowing)
    // 2 кейс: Тип спарка не будет указан при инициализации
    // 3 кейс: Используется несколько спарков в выражении (spark!() + spark!())
    // 
    // Также спарк не определится если не будет в statement::local, поэтому условная
    // инициализация не работает для спарка
    pub is_spark: bool,

    // Явлется ли эта переменная мутабельной
    pub is_mut: bool,
}

impl Variable {
    pub fn new() -> Self {
        Self {
            // NO_TYPE как константа отсуствия типа
            variable_type: NO_TYPE.to_string(),

            // Заглушки, во время парсинга станет точно ясна явлется ли переменная
            // реактивной/мутабельной
            is_spark: false,

            // Немутабельна по дефолту
            is_mut: false,
        }
    }
}

/// Текущая область видимости, хранить всю таблицу символов для этой области. Начинается
/// с { и при входе в эту область видимости экземпляр этой структуры будет скопирован
/// чтобы когда произойдёт выход из неё все созданные в ней имена были заменены
/// состояние слепок которого был сделан до входа в область видимости
#[derive(Debug, Clone)]
pub struct Scope {
    pub variables: HashMap<String, Variable>,
    pub screen_index: usize,
    pub depth: usize,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            // Нет имён на старте
            variables: HashMap::new(),
            screen_index: 0,
            depth: 0,
        }
    }

    /// Метод для проверки есть ли в хэш мапе имён области видимости переменная с
    /// таким именем и is_spark = true
    pub fn is_spark(&self, name: &str) -> bool {
        if let Some(variable) = self.variables.get(name) {
            return variable.is_spark;
        }

        // Если такой переменной вообще нет то это тоже означает false
        false
    }
}

/// Главная структура анализатора для которого реализуется Visitor и который выполняет
/// роль анализа кода пользователя firework чтобы построить граф реактивности и
/// валидировать правильное использование спарков
pub struct Analyzer {
    // Ошибки компиляции, они накапливаются весь парсинг чтобы по завершению анализа
    // вывести их в терминал. Подробнее про сообщения ошибок можно узнать в файле
    // firework_macro/src/compiler/errors.rs. Все ошибки начинаются с FE, то есть
    // Firework Error и заканчиваются числом из трёх цифр, это номер ошибки. Пример:
    // FE001, FE004
    pub errors: Vec<Error>,
    
    // Выходные токены
    pub output: TokenStream,
    
    // Область видимости
    pub scope: Scope,

    // Statement это блок кода от начала до ; фигурных скобок или в некоторых случаях
    // запятой. Нужно точно знать на каком statement мы сейчас. На старте это 1, поэтому
    // итерацию нужно начать с единицы
    pub statement_index: usize,

    // Флаг который означает был ли использован функциональный виджет layout! в текущем
    // лайаут блоке. Описывать лайаут можно только один раз в лайаут блоке
    pub descript_layout: bool,

    // 
    pub statement: FireworkStatement,
    pub ir: FireworkIR,
    pub function_name: Option<String>,

    // Буферы
    // Буфер который используется для хранения текущего типа в парсинге переменной,
    // если типа не указан то используется значения константы NO_TYPE
    pub current_type: String,

    // Временный вектор имён переменных которые были найдены в текущем let, но ещё
    // не добавленных в scope.variables
    pending_vars: Vec<(String, Variable)>,

    // Маркер который показывает явлется ли этот statement частью реактивного блока.
    // Реактивный блок это условие, цикл (for/while) или match в условии которого
    // используется спарк. Если None то команда не в реактивном блоке, если Some(usize)
    // то строка в реактивном блоке, а usize это начало блока. Вложенные конструкции и
    // вложенные реактивные блоки не меняют этот флаг, он всегда показывает на первый
    // реактивный блок. Второе значение кортежа это цикл (нужен ли микрорантайм)
    reactive_block: Option<(usize, bool)>,

    // Счётчики чтобы генерировать названия полей глобальной структуры экрана
    widget_counter: usize,
    spark_counter: usize,

    pub block_id: u64,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            // При старте нет ошибок
            errors: Vec::new(),
            
            output: TokenStream::new(),
            scope: Scope::new(),

            // Нулевая команда
            statement_index: 0,

            descript_layout: false,

            statement: FireworkStatement {
                action: FireworkAction::DefaultCode,
                is_reactive_block: false,
                index: 0,
                screen_name: String::from(""),
                scope: Scope::new(),
                string: String::from(""),
            },

            ir: FireworkIR {
                statements: Vec::new(),
                screen_structs: HashMap::new(),
                screens: Vec::new(),
                items: Vec::new(),
            },

            function_name: None,

            current_type: String::from(NO_TYPE),
            pending_vars: Vec::new(),

            // Изначально мы не в реактивном блоке
            reactive_block: None,

            // Счётчики
            widget_counter: 0,
            spark_counter: 0,

            block_id: 0,
        }
    }

    /// Метод для вывода всего что собранно в области видимости
    pub fn log_scope(&self) {
        // println!("{:#?}", self.scope.variables);
    }
   
    /// Метод обёртка над SparkFinder чтобы быстро найти наличие спарка в выражении
    /// используется в коде чтобы проверить явлется ли блок реактивным
    pub fn has_spark(&self, expr: &Expr) -> bool {
        let mut found = Vec::new();

        let mut finder = SparkFinder {
            scope: &self.scope,
            found: &mut found,
        };

        finder.visit_expr(&expr);
        
        !found.is_empty()
    }

    pub fn get_sparks(&self, expr: &Expr) -> Vec<String> {
        let mut found = Vec::new();

        let mut finder = SparkFinder {
            scope: &self.scope,
            found: &mut found,
        };

        finder.visit_expr(&expr);
        
        found
    }

    /// Добавляет поле в структуру экрана, если экран ещё не зарегистрирован в FireworkIR
    /// то он создаётся там
    pub fn add_field_to_struct(&mut self, field_name: String, field_type: String) {
        if let Some(function_name) = &self.function_name {
            // Добавляет значение в вектор (описание структуры экрана), если такого
            // значения нет в хэш мапе то создаёт пустой вектор
            self.ir.screen_structs.entry(format!("ApplicationUiBlockStruct{}", self.block_id.to_string()))
                .or_insert_with(Vec::new)
                .push((field_name, field_type));
        }
    }

    fn handle_reactive_block(
        &mut self,
        sparks: Vec<String>,
        is_loop: bool,
        open_code: String, 
        action: FireworkAction,
        visit_fn: impl FnOnce(&mut Self),
    ) {
        self.scope.depth += 1;

        let state = self.reactive_block;
        let condition_has_spark = !sparks.is_empty();
        
        let mut open_statement = self.statement.clone();
        open_statement.string = open_code;
        
        if condition_has_spark && self.reactive_block.is_none() {
            open_statement.action = action;
            self.reactive_block = Some((self.statement_index, is_loop));
        } else {
            open_statement.action = FireworkAction::DefaultCode;
        }
        
        self.ir.statements.push(open_statement);
        self.statement_index += 1;
        
        let saved_action = self.statement.action.clone();
        self.statement.action = FireworkAction::DefaultCode;
        
        visit_fn(self);
        
        self.statement.action = FireworkAction::DefaultCode;
        self.statement.string = "}".to_string();
        self.statement_index += 1; 
        self.reactive_block = state;

        self.scope.depth -= 1;
    }
}

impl<'ast> Visit<'ast> for Analyzer {
    // Генерирует заглушки для функций чтобы компилятор не выдал ошибку "функция отсуствует"
    // вероятно это временное решение
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let mut function_head = String::from("");
        for attr in &node.attrs {
            function_head.push_str(format!("{}\n", quote::quote! { #attr }).as_str()); 
        }
        
        let vis = &node.vis;
        let constness = &node.sig.constness;
        let asyncness = &node.sig.asyncness;
        let unsafety = &node.sig.unsafety;
        let abi = &node.sig.abi;
        let fn_token = &node.sig.fn_token;
        let ident = &node.sig.ident;
        let generics = &node.sig.generics;
        let inputs = &node.sig.inputs;
        let output = &node.sig.output;
        
        let signature = quote::quote! {
            #vis #constness #asyncness #unsafety #abi #fn_token #ident #generics (#inputs) #output
        };

        function_head.push_str(format!("{}", signature).as_str());

        let mut fn_stub = node.clone();
        fn_stub.block = syn::parse2(quote::quote! {
            {}
        }).expect("Failed to parse item"); 
        
        self.output.extend(quote::quote! {
            #fn_stub
        });

        // Добавление всех аргументов в область видимости как переменных
        for input in &node.sig.inputs {
            self.visit_fn_arg(input);
        }

        let function_name = node.sig.ident.to_string();
        self.function_name = Some(function_name.clone());
        self.ir.screens.push((function_name.clone(), function_head, self.scope.screen_index));
        self.statement.screen_name = function_name;

        syn::visit::visit_item_fn(self, node);

        self.scope.screen_index += 1;
    }

    fn visit_fn_arg(&mut self, i: &'ast FnArg) {
        if let FnArg::Typed(pat_type) = i {
            self.current_type = pat_type.ty.to_token_stream().to_string();
        
            // Переменные будут добавляться в pending_vars
            self.visit_pat(&pat_type.pat);
        
            for (name, mut var_data) in self.pending_vars.drain(..) {
                // Аргумент функции не может быть спарком
                var_data.is_spark = false;

                self.scope.variables.insert(name, var_data);
            }

            self.current_type = String::from(NO_TYPE);
        }
    }

    fn visit_local(&mut self, i: &'ast Local) {
        // Очистка данных из старого let
        self.pending_vars.clear();
        visit::visit_pat(self, &i.pat);

        // Найден ли спарк в правой части
        let mut found_spark = false;

        if let Some(local_init) = &i.init {
            // Валидатор сам сделает подсчёт спарков в выражении
            let mut validator = SparkValidator {
                spark_count: 0,
            };

            // Вызов из валидатора
            validator.visit_expr(&local_init.expr);

            if validator.spark_count > 1 {
                // FE006 нельзя делать выражения с несколькими инициализациями спарков
                self.errors.push(compile_error_spanned(
                    &local_init.expr,
                    SPARK_MULTIPLE_ERROR,
                ));
            }

            // SparkValidator нашёл один спарк в выражении
            if validator.spark_count == 1 {
                found_spark = true;
            }

            // Временный вектор чтобы сложить туда поля, так как пушить нельзя из-за
            // мутабельной ссылки от drain
            let mut temp_fields_to_struct: Vec<(String, String)> = Vec::new();
            for (name, mut var_data) in self.pending_vars.drain(..) {
                var_data.is_spark = found_spark;
 
                if found_spark {
                    temp_fields_to_struct.push((
                        format!("spark_{}", self.spark_counter),
                        var_data.clone().variable_type,
                    ));

                    self.spark_counter += 1;

                    // FE002, нельзя затенять существующую переменную спарком
                    if self.scope.variables.contains_key(&name) {
                        self.errors.push(compile_error_spanned(
                            &i.pat,
                            SPARK_SHADOWING_ERROR,
                        ));
                    }

                    // FE003, у спарка должен быть тип данных, например u32
                    // let mut spark1: u32 = spark!(10); 
                    if var_data.variable_type == NO_TYPE.to_string() {
                        self.errors.push(compile_error_spanned(
                            &i.pat,
                            SPARK_TYPE_ERROR,
                        ));
                    }

                    self.spark_counter += 1;
                    self.statement.action = FireworkAction::InitialSpark(
                        name.clone(), 0, var_data.clone().variable_type,
                    );
                }

                // FE004, нельзя затенить спарк
                if let Some(value) = self.scope.variables.get(&name) {
                    if value.is_spark { 
                        self.errors.push(compile_error_spanned(
                            &i.pat,
                            SPARK_UNIQUE_NAME_ERROR,
                        ));
                    }
                }

                self.scope.variables.insert(name, var_data);
            }

            for (field_name, field_type) in temp_fields_to_struct.iter() {
                self.add_field_to_struct(field_name.to_string(), field_type.to_string())
            }
        }

        // Вызов visit_local в конце не нужен
    }

    fn visit_pat_type(&mut self, i: &'ast PatType) {
        // Это строка нужна чтобы запомнить текущий тип дпнных, это будет нужно в ветке
        // ident для определения типа, что потребуется в других ветках
        self.current_type = i.ty.to_token_stream().to_string();

        visit::visit_pat_type(self, i);

        // После завершения обработки нужно сбросить буфер
        self.current_type = String::from(NO_TYPE);
    }

    fn visit_pat_ident(&mut self, i: &'ast PatIdent) {
        self.pending_vars.push((i.ident.to_string(), Variable {
            variable_type: self.current_type.clone(),
            is_mut: i.mutability.is_some(),
            is_spark: false,
        }));

        // На всякий случай
        visit::visit_pat_ident(self, i);
    }

    // Вход в новую область видимости
    fn visit_block(&mut self, i: &'ast syn::Block) {
        // Сначала клонируем всё состояние текущей области видимости, когда эта область
        // видимости закончится (та, что сейчас открывается) все переменные и не только
        // созданные внутри неё будут дропнуты и мы не можем их использовать. После
        // завершения блока нам нужно вернуть ранее сохранённое состояние, а для этого
        // мы будем использовать клон который создаётся здесь
        let scope = self.scope.clone();

        // Парсинг области видимости, переменные созданные в этой области видимости будут
        // в self.scrope.variables
        visit::visit_block(self, i);

        // Дебаг вывод всех переменных который собрали в этой новой области видимости
        self.log_scope();

        // Область видимости закончилась, нужно восстановить состояние используя клон
        self.scope = scope;
    }

    /// Макрос который используются не в выражении, а как отдельный statement (команда)
    fn visit_macro(&mut self, i: &'ast Macro) {
        let name = i.path.to_token_stream().to_string();

        // Дебаг
        // println!("{}", name);

        // Проверка что лайаут конфигурируется только один раз в лайаут блоке
        if name == "layout" {
            if self.descript_layout {
                // FE009
                self.errors.push(compile_error_spanned(
                    // Весь макрос
                    i,

                    LAYOUT_MULTIPLE_ERROR,
                ));

                return;
            }

            self.descript_layout = true;
        }
       
        // Лайаут это конструкция layout_name! { // Обычный раст код }; все токены
        // внутри нужно распарсить как обычный раст код через block (Не file и не expr)
        // установка настроек лайаута идёт через специальный функциональный виджет
        // layout!(...); сам лайаут это лишь контейнер для виджетов. Такой подход
        // позволяет оставить DSL только на уровне виджетов, а весь остальной код
        // держать как чистый раст
        if is_layout(&name) {
            let inner_tokens = &i.tokens;

            let parse_result: syn::Result<Block> = syn::parse2(quote::quote! {
                {
                    #inner_tokens
                }
            });

            if let Ok(block) = parse_result {
                // До входа в новый лайаут блок снимает флаг чтобы при конфигурации
                // нового лайаута не было FE009, делаем это до прохода по командам
                // внутри лайаут блока
                self.descript_layout = false;

                let mut has_microruntime = false;
                if let Some((start, need_microruntime)) = self.reactive_block {
                    has_microruntime = need_microruntime;
                }

                self.statement.action = FireworkAction::LayoutBlock(
                    name.clone(), has_microruntime,
                );
                
                self.statement.scope = self.scope.clone(); 
                self.ir.statements.push(self.statement.clone()); 

                self.scope.depth += 1;
                self.statement.scope.depth += 1;

                for statement in block.stmts { 
                    // Парсинг всех команд внутри
                    self.visit_stmt(&statement);
                }
               
                self.scope.depth -= 1;
                self.statement.scope.depth -= 1;
                self.statement.action = FireworkAction::DefaultCode;
                self.statement.string = "}".to_string();
                self.statement_index += 1;

                self.ir.statements.push(self.statement.clone());
            } else {
                // FE008, невалидный синтаксис в лайауте. Как уже было сказанно ранее,
                // лайаут требует полностью валидный раст синтаксис
                self.errors.push(compile_error_spanned(
                    i.tokens.clone(),
                    LAYOUT_PARSE_ERROR,
                ));

                return;
            }
        } else if is_widget(&name) {
            // Виджет это строительный блок ui, имеет синтаксис widget_name!(field: 123);
            // в отличии от лайаута там не валидный rust код, а специальный DSL который
            // похож на конструкцию структур. Пример:
            //
            // rect! (
            //  image: "test.png".to_string(),
            // ); 
            let args: WidgetArgs = match syn::parse2(i.tokens.clone()) {
                Ok(args) => args,
                Err(e) => {
                    // Ошибка FE007, нарушение синтаксиса DSL виджета. Синтаксис только
                    //
                    // widget_name!(
                    //   field1: 10,
                    //   field2: 20,
                    // );
                    self.errors.push(compile_error_spanned(
                        i.tokens.clone(),
                        WIDGET_PARSE_ERROR, 
                    ));

                    // Выход чтобы не продолжать
                    return;
                }
            };

            let mut fields_map_spark: HashMap<String, Vec<String>> = HashMap::new();

            for prop in args.properties {
                let prop_name = prop.name.to_string();
                let mut sparks_in_this_field = Vec::new();
                
                let mut finder = SparkFinder {
                    scope: &self.scope,
                    found: &mut sparks_in_this_field,
                };

                finder.visit_expr(&prop.value); 

                if sparks_in_this_field.is_empty() {
                    fields_map_spark.insert(prop_name, Vec::new()); 
                } else {
                    fields_map_spark.insert(prop_name, sparks_in_this_field); 
                }
            } 

            if let Some(skin) = map_skin(&name) {
                self.add_field_to_struct(
                    format!("widget_object_{}", self.widget_counter),
                    skin,
                );
            }

            // [REFACTORME]
            // Убрать дубляж кода
            let mut has_microruntime = false;
            if let Some((start, need_microruntime)) = self.reactive_block {
                has_microruntime = need_microruntime;
            }

            self.statement.action = FireworkAction::WidgetBlock(
                name.clone(), fields_map_spark, has_microruntime, self.widget_counter,
            );

            self.widget_counter += 1; 
        }

        visit::visit_macro(self, i);
    }

    /// Присваивание значения к переменной которая инициализирована как спарк считаетсч
    /// обновлением состояния и требует обновления UI
    fn visit_expr_assign(&mut self, i: &'ast ExprAssign) {
        if let Some(root_name) = get_root_variable_name(&i.left) {
            if let Some(variable) = self.scope.variables.get(&root_name) {
                if variable.is_spark {
                    self.statement.action = FireworkAction::UpdateSpark(root_name);
                }
            }
        }

        visit::visit_expr_assign(self, i);
    }

    /// Кейс обновления состояния для бинарных операций, например spark += 1 или
    /// spark %= 2, также требует обновления ui
    fn visit_expr_binary(&mut self, i: &'ast ExprBinary) {
        let is_mutation = match i.op {
            BinOp::AddAssign(_)   | BinOp::SubAssign(_)    | BinOp::MulAssign(_)    |
            BinOp::DivAssign(_)   | BinOp::RemAssign(_)    | BinOp::BitAndAssign(_) |
            BinOp::BitOrAssign(_) | BinOp::BitXorAssign(_) | BinOp::ShlAssign(_)    |
            BinOp::ShrAssign(_)  => true,

            _ => false,
        };

        if is_mutation {
            if let Some(root_name) = get_root_variable_name(&i.left) {
                if let Some(variable) = self.scope.variables.get(&root_name) {
                    if variable.is_spark {
                        self.statement.action = FireworkAction::UpdateSpark(root_name);
                    }
                }
            }
        }

        visit::visit_expr_binary(self, i);
    }

    fn visit_expr_method_call(&mut self, i: &'ast syn::ExprMethodCall) {
        if let Some(root_name) = get_root_variable_name(&i.receiver) {
            if let Some(variable) = self.scope.variables.get(&root_name) {
                if variable.is_spark {
                    let method_name = i.method.to_string();

                    // Только мутабельные методы, узнать это можно по типу спарка
                    // через хелпер, если это кастомный тип то используется хак и
                    // все методы считаются мутабельными
                    if is_mutable_method(&variable.variable_type, &method_name) {
                        self.statement.action = FireworkAction::UpdateSpark(root_name);
                    }
                }
            }
        }
    }

    fn visit_stmt(&mut self, i: &'ast Stmt) {
        let mut layout_name = "".to_string();
        let should_push = if let Stmt::Macro(stmt_macro) = i {
            layout_name = stmt_macro.mac.path.to_token_stream().to_string();
            !is_layout(&layout_name)
        } else {
            true
        };

        if should_push {
            self.statement.string = i.to_token_stream().to_string();
        } else {
            self.statement.string = format!("{} {{", layout_name);
        }

        // println!("STATEMENT: {}", self.statement_index);
        if let Some(root_id) = self.reactive_block {
            // println!("Statement {} is reactive, start: {}", self.statement_index, root_id.0);
            self.statement.is_reactive_block = true;
        }
        
        visit::visit_stmt(self, i); 
        
        self.statement_index += 1; 
        
        if should_push {
            // Если это лайаут блок то клонирование области видимости и пуш уже
            // были и клонировать второй раз нет смысла
            self.statement.scope = self.scope.clone();
            self.ir.statements.push(self.statement.clone());
        }
        
        self.statement.index = self.statement_index;
        self.statement.action = FireworkAction::DefaultCode;
        self.statement.is_reactive_block = false;
    }

    fn visit_expr_if(&mut self, i: &'ast ExprIf) {
        let sparks = self.get_sparks(&i.cond);
        let condition_code = i.cond.to_token_stream().to_string();
        
        self.handle_reactive_block(
            sparks.clone(),
            false,
            format!("if {} {{", condition_code),
            FireworkAction::ReactiveIf(sparks),
            |this| visit::visit_expr_if(this, i),
        );
    }

    fn visit_expr_while(&mut self, i: &'ast ExprWhile) {
        let sparks = self.get_sparks(&i.cond);
        let condition_code = i.cond.to_token_stream().to_string();
        
        self.handle_reactive_block(
            sparks.clone(),
            true,
            format!("while {} {{", condition_code),
            FireworkAction::ReactiveWhile(sparks.clone()),
            |this| visit::visit_expr_while(this, i),
        );
    }
    
    fn visit_expr_for_loop(&mut self, i: &'ast ExprForLoop) {
        let sparks = self.get_sparks(&i.expr);
        let pattern_code = i.pat.to_token_stream().to_string();
        let expr_code = i.expr.to_token_stream().to_string();
        
        self.handle_reactive_block(
            sparks.clone(),
            true,
            format!("for {} in {} {{", pattern_code, expr_code),
            FireworkAction::ReactiveFor(sparks.clone()),
            |this| visit::visit_expr_for_loop(this, i),
        );
    }
    
    fn visit_expr_match(&mut self, i: &'ast ExprMatch) {
        let sparks = self.get_sparks(&i.expr);
        let expr_code = i.expr.to_token_stream().to_string();
        
        self.handle_reactive_block(
            sparks.clone(),
            false,
            format!("match {} {{", expr_code),
            FireworkAction::ReactiveMatch(sparks.clone()),
            |this| visit::visit_expr_match(this, i),
        );
    }

    fn visit_expr_loop(&mut self, i: &'ast ExprLoop) {
        self.handle_reactive_block(
            Vec::new(),
            true,
            "loop {".to_string(),
            FireworkAction::DefaultCode,
            |this| visit::visit_expr_loop(this, i),
        );
    }

    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        self.ir.items.push(node.to_token_stream().to_string());
        self.output.extend(node.to_token_stream());
    }
    
    fn visit_item_enum(&mut self, node: &'ast ItemEnum) {
        self.ir.items.push(node.to_token_stream().to_string());
        self.output.extend(node.to_token_stream());
    }
    
    fn visit_item_type(&mut self, node: &'ast ItemType) {
        self.ir.items.push(node.to_token_stream().to_string());
        self.output.extend(node.to_token_stream());
    }
    
    fn visit_item_trait(&mut self, node: &'ast ItemTrait) {
        self.ir.items.push(node.to_token_stream().to_string());
        self.output.extend(node.to_token_stream());
    }
    
    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        self.ir.items.push(node.to_token_stream().to_string());
        self.output.extend(node.to_token_stream());
    }
    
    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        self.ir.items.push(node.to_token_stream().to_string());
        self.output.extend(node.to_token_stream());
    }
    
    fn visit_item_use(&mut self, node: &'ast ItemUse) {
        self.ir.items.push(node.to_token_stream().to_string());
        self.output.extend(node.to_token_stream());
    }
    
    fn visit_item_extern_crate(&mut self, node: &'ast ItemExternCrate) {
        self.ir.items.push(node.to_token_stream().to_string());
        self.output.extend(node.to_token_stream());
    }
    
    fn visit_item_foreign_mod(&mut self, node: &'ast ItemForeignMod) {
        self.ir.items.push(node.to_token_stream().to_string());
        self.output.extend(node.to_token_stream());
    }
    
    fn visit_item_macro(&mut self, node: &'ast ItemMacro) {
        self.ir.items.push(node.to_token_stream().to_string());
        self.output.extend(node.to_token_stream());
    }
}

pub fn prepare_tokens(tokens: Vec<TokenTree>, id: u64) -> (proc_macro2::TokenStream, Option<proc_macro2::TokenStream>, Option<FireworkIR>) {
    let token_stream: proc_macro2::TokenStream = tokens.into_iter().collect();
    
    let file = match syn::parse2::<File>(token_stream) {
        Ok(file) => file,
        Err(e) => return (proc_macro2::TokenStream::new(), Some(e.to_compile_error()), None),
    };
    
    let mut analyzer = Analyzer::new();
    analyzer.block_id = id;
    analyzer.visit_file(&file); 

    println!("IR len: {}, IR: {:#?}", analyzer.ir.statements.len(), analyzer.ir);
    
    if !analyzer.errors.is_empty() {
        let mut final_error = analyzer.errors[0].clone();
        
        for error in analyzer.errors.iter().skip(1) {
            final_error.combine(error.clone());
        }

        (analyzer.output, Some(final_error.to_compile_error()), Some(analyzer.ir))
    } else {
        (analyzer.output, None, Some(analyzer.ir))
    }
}
