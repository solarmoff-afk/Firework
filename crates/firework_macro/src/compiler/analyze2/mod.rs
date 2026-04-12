// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

mod marks;
mod spark;
mod widget;
mod visitor;
mod type_inference;

mod context;
mod lifetime_manager;

use proc_macro2::{TokenTree, TokenStream, Span};
use syn::*;
use syn::visit::Visit;
use std::collections::HashMap;
use quote::ToTokens;

use widget::{is_widget, is_layout, map_skin, WidgetArgs};
use spark::{SparkValidator, SparkFinder, SparkFinderWithId, get_root_variable_name};
use context::AnalyzeContext;
use lifetime_manager::{Variable, Scope, LifetimeManager};

use crate::compiler::utils::is_mutable_method;
use crate::compiler::codegen::actions::{
    FireworkIR, FireworkStatement, FireworkAction, FireworkWidgetField, FireworkReactiveBlock,
};
use crate::compiler::error::*;

/// Нельзя хранить String поэтому используется &str, при использовании нужно использовать
/// String::from, но это позволяет не тянуть lazy_static или другой крейт
pub const NO_TYPE: &str = "NO TYPE";

/// Главная структура анализатора для которого реализуется Visitor и который выполняет
/// роль анализа кода пользователя firework чтобы построить граф реактивности и
/// валидировать правильное использование спарков
pub struct Analyzer {
    pub context: AnalyzeContext,

    pub lifetime_manager: LifetimeManager,

    // Statement это блок кода от начала до ; фигурных скобок или в некоторых случаях
    // запятой. Нужно точно знать на каком statement мы сейчас. На старте это 1, поэтому
    // итерацию нужно начать с единицы
    statement_index: usize,

    // Флаг который означает был ли использован функциональный виджет layout! в текущем
    // лайаут блоке. Описывать лайаут можно только один раз в лайаут блоке
    descript_layout: bool,

    // Текущее имя функции, если мы вне функции то None
    function_name: Option<String>,

    // Буферы
    // Буфер который используется для хранения текущего типа в парсинге переменной,
    // если типа не указан то используется None
    current_type: String,

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
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            context: AnalyzeContext::new(),
            lifetime_manager: LifetimeManager::new(),

            // Нулевая команда
            statement_index: 0,

            // Был ли настроен текущий лайаут, этот флаг нужен чтобы исключить двойной
            // вызов функционального виджета layout
            descript_layout: false, 

            function_name: None,

            current_type: String::from(NO_TYPE),
            pending_vars: Vec::new(),

            // Изначально мы не в реактивном блоке
            reactive_block: None, 
        }
    }

    /// Метод для вывода всего что собранно в области видимости
    pub fn log_scope(&self) {
        #[cfg(feature = "debug_output")]
        println!("{:#?}", self.lifetime_manager.scope.variables);
    }
   
    /// Метод обёртка над SparkFinder чтобы быстро найти наличие спарка в выражении
    /// используется в коде чтобы проверить явлется ли блок реактивным и получить вектор
    /// спарков который содержит кортеж (имя, айди)
    pub fn get_sparks(&self, expr: &Expr) -> Vec<(String, usize)> {
        let mut found = Vec::new();

        let mut finder = SparkFinderWithId {
            scope: &self.lifetime_manager.scope,
            found: &mut found,
        };

        finder.visit_expr(&expr);
        
        found
    }

    /// Добавляет поле в структуру экрана, если экран ещё не зарегистрирован в FireworkIR
    /// то он создаётся там
    pub fn add_field_to_struct(&mut self, field_name: String, field_type: String) {
        if let Some(_function_name) = &self.function_name {
            // Добавляет значение в вектор (описание структуры экрана), если такого
            // значения нет в хэш мапе то создаёт пустой вектор
            self.context.ir.screen_structs.entry(
                format!("ApplicationUiBlockStruct{}",
                    self.lifetime_manager.scope.screen_index.to_string()))
                .or_insert_with(Vec::new)
                .push((field_name, field_type));
        }
    }

    /// Функция хэлпер для регистрации реактивного блока в IR. Реактивный блок это блок
    /// (условие, цикл, match) которые содержит реактивную переменную (спарк) в своём
    /// условии. Он забирает всё содержимое тело поэтому реактивный блок в реактином блоке
    /// не считается отдельным реактивным блоком. Также он вызывает visit метод через
    /// замыкание (visit_fn). Добавляет закрывающий блок (}) в конце блока
    fn handle_reactive_block(
        &mut self,
        sparks: Vec<(String, usize)>,
        is_loop: bool,
        open_code: String, 
        action: FireworkAction,
        visit_fn: impl FnOnce(&mut Self),
    ) {
        // Добавление к счётчику глубины. Это используется для форматирования вывода чтобы
        // определить сколько табов нужно
        self.lifetime_manager.scope.depth += 1;

        // Текущее состояние
        let state = self.reactive_block;
                
        // Стейтемент для открытия реактивного блока чтобы кодогенератор мог правильно
        // сгенерировать реактивный блок
        let mut open_statement = self.context.statement.clone();
        open_statement.string = open_code;

        // Нулевой эффект это эффект который не содержит спарков в условии. Он нужен чтобы
        // создать код который выполняется только при билде и навигации, а Event или
        // Reactive флэши его не трогают
        let mut is_null_effect = false;
        let condition_has_spark = !sparks.is_empty();
        
        // Если это эффект
        if let FireworkAction::ReactiveBlock(FireworkReactiveBlock::Effect, vec) = &action {
            // Нулевой эффект должен быть пустым
            is_null_effect = vec.is_empty();
        }
        
        // Если в условии есть спарки ИЛИ это нулевой эффект и мы сейчас не в реактивном
        // блоке то мы входим в реактивный блок. Реактивные блоки в реактивных блоках не
        // работают. То есть реактивный блок будет создан если в условии есть спарки или
        // если это эффект без спарков. Если это эффект у которого есть спарки то это
        // сделает true condition_has_spark, а если это эффект без спарков то is_null_effect
        if (condition_has_spark || is_null_effect) && self.reactive_block.is_none() {
            open_statement.action = action;
            open_statement.is_reactive_block = true;

            self.reactive_block = Some((self.statement_index, is_loop));
        } else {
            // Иначе это может быть else реактивного блока
            open_statement.action = FireworkAction::ReactiveElse;

            // Если это не else то обычный код
            if !matches!(action, FireworkAction::ReactiveElse) {
                open_statement.action = FireworkAction::DefaultCode;
            }

            open_statement.is_reactive_block = false;
        } 

        // Открывающий стейтемент реактивного блока
        self.context.ir.statements.push(open_statement);
        self.statement_index += 1;
        
        // let _saved_action = self.statement.action.clone();
        self.context.statement.action = FireworkAction::DefaultCode;
        
        // Замыкание чтобы выполнить все блоки, self передаётся из-за того что в
        // расте нельзя использовать self внутри метода этой же структуры поэтому
        // здесь передаётся self как аргумент замыкания
        visit_fn(self);
        
        // Закрывающий стейтемент реактивного блока
        self.context.statement.action = FireworkAction::ReactiveBlockTerminator;
        self.context.statement.string = "}".to_string();
        self.statement_index += 1;

        // Закрывающая фигурная скобка также является частью реактивного блока
        self.context.statement.is_reactive_block = true;
        
        self.reactive_block = state;
        
        // Защита от переполнения
        if self.lifetime_manager.scope.depth > 0 {
            self.lifetime_manager.scope.depth -= 1;
        }
    }

    pub fn update_scope(&mut self, scope: Scope, set_scope: bool) {
        let base_stmt = self.context.statement.clone();
        let drop_statements = self.lifetime_manager.update_scope(scope, set_scope, &base_stmt);
        
        for stmt in drop_statements {
            self.context.ir.statements.push(stmt);
            self.statement_index += 1;
        }
    }
}

impl<'ast> Visit<'ast> for Analyzer {
    /// Генерирует заглушки для функций чтобы компилятор не выдал ошибку "функция отсуствует"
    /// вероятно это временное решение. Также собирает сигнатуру функции для кодогенератора
    fn visit_item_fn(&mut self, i: &'ast ItemFn) {
        self.analyze_item_fn(i);
    }

    fn visit_fn_arg(&mut self, i: &'ast FnArg) {
        self.analyze_fn_arg(i);
    }

    fn visit_local(&mut self, i: &'ast Local) {
        self.analyze_local(i);
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
            spark_id: 0, // HARDCOD
        }));

        // На всякий случай
        visit::visit_pat_ident(self, i);
    }

    /// Вход в новую область видимости
    fn visit_block(&mut self, i: &'ast syn::Block) {
        self.analyze_block(i);
    }

    /// Макрос который используются не в выражении, а как отдельный statement (команда)
    fn visit_macro(&mut self, i: &'ast Macro) {
        self.analyze_macro(i);
    }

    /// Присваивание значения к переменной которая инициализирована как спарк считаетсч
    /// обновлением состояния и требует обновления UI
    fn visit_expr_assign(&mut self, i: &'ast ExprAssign) {
        self.analyze_expr_assign(i);
    }

    /// Кейс обновления состояния для бинарных операций, например spark += 1 или
    /// spark %= 2, также требует обновления ui
    fn visit_expr_binary(&mut self, i: &'ast ExprBinary) {
        self.analyze_expr_binary(i);
    }

    fn visit_expr_method_call(&mut self, i: &'ast syn::ExprMethodCall) {
        self.analyze_expr_method_call(i);
    }

    fn visit_stmt(&mut self, i: &'ast Stmt) {
        self.analyze_stmt(i);
    }

    fn visit_expr_if(&mut self, i: &'ast ExprIf) {
        self.analyze_expr_if(i);
    }

    fn visit_expr_while(&mut self, i: &'ast ExprWhile) {
        self.analyze_expr_while(i);
    }
    
    fn visit_expr_for_loop(&mut self, i: &'ast ExprForLoop) {
        self.analyze_expr_for_loop(i);
    }
    
    fn visit_expr_match(&mut self, i: &'ast ExprMatch) {
        self.analyze_expr_match(i);
    }

    fn visit_expr_loop(&mut self, i: &'ast ExprLoop) {
        self.analyze_expr_loop(i);
    }

    fn visit_expr_break(&mut self, i: &'ast ExprBreak) {
        self.analyze_expr_break(i);
    }

    fn visit_expr_continue(&mut self, i: &'ast ExprContinue) {
        self.analyze_expr_continue(i);
    }

    fn visit_expr_return(&mut self, i: &'ast ExprReturn) {
        self.analyze_expr_return(i);
    }

    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        self.context.ir.items.push(node.to_token_stream().to_string());
        self.context.output.extend(node.to_token_stream());
    }
    
    fn visit_item_enum(&mut self, node: &'ast ItemEnum) {
        self.context.ir.items.push(node.to_token_stream().to_string());
        self.context.output.extend(node.to_token_stream());
    }
    
    fn visit_item_type(&mut self, node: &'ast ItemType) {
        self.context.ir.items.push(node.to_token_stream().to_string());
        self.context.output.extend(node.to_token_stream());
    }
    
    fn visit_item_trait(&mut self, node: &'ast ItemTrait) {
        self.context.ir.items.push(node.to_token_stream().to_string());
        self.context.output.extend(node.to_token_stream());
    }
    
    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        self.context.ir.items.push(node.to_token_stream().to_string());
        self.context.output.extend(node.to_token_stream());
    }
    
    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        self.context.ir.items.push(node.to_token_stream().to_string());
        self.context.output.extend(node.to_token_stream());
    }
    
    fn visit_item_use(&mut self, node: &'ast ItemUse) {
        self.context.ir.items.push(node.to_token_stream().to_string());
        self.context.output.extend(node.to_token_stream());
    }
    
    fn visit_item_extern_crate(&mut self, node: &'ast ItemExternCrate) {
        self.context.ir.items.push(node.to_token_stream().to_string());
        self.context.output.extend(node.to_token_stream());
    }
    
    fn visit_item_foreign_mod(&mut self, node: &'ast ItemForeignMod) {
        self.context.ir.items.push(node.to_token_stream().to_string());
        self.context.output.extend(node.to_token_stream());
    }
    
    fn visit_item_macro(&mut self, node: &'ast ItemMacro) {
        self.context.ir.items.push(node.to_token_stream().to_string());
        self.context.output.extend(node.to_token_stream());
    }
}

pub fn prepare_tokens(tokens: Vec<TokenTree>, _id: u64) -> (proc_macro2::TokenStream, Option<proc_macro2::TokenStream>, Option<FireworkIR>) {
    let token_stream: proc_macro2::TokenStream = tokens.into_iter().collect();
    
    let file = match syn::parse2::<File>(token_stream) {
        Ok(file) => file,
        Err(e) => return (proc_macro2::TokenStream::new(), Some(e.to_compile_error()), None),
    };
    
    let mut analyzer = Analyzer::new();
    analyzer.lifetime_manager.scope.screen_index_generate();
    analyzer.visit_file(&file); 

    #[cfg(feature = "debug_output")]
    println!("IR len: {}, IR: {:#?}", analyzer.context.ir.statements.len(), analyzer.context.ir);
    
    if !analyzer.context.errors.is_empty() {
        let mut final_error = analyzer.context.errors[0].clone();
        
        for error in analyzer.context.errors.iter().skip(1) {
            final_error.combine(error.clone());
        }

        (analyzer.context.output, Some(final_error.to_compile_error()), Some(analyzer.context.ir))
    } else {
        (analyzer.context.output, None, Some(analyzer.context.ir))
    }
}
