// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

mod marks;
mod spark;
mod widget;
mod visitor;
mod type_checker;

use proc_macro2::{TokenTree, TokenStream};
use syn::*;
use syn::visit::Visit;
use std::collections::HashMap;
use quote::ToTokens;
use rand::Rng;

use marks::*;
use widget::{is_widget, is_layout, map_skin, WidgetArgs};
use spark::{SparkValidator, SparkFinder, SparkFinderWithId, get_root_variable_name};

use crate::compiler::utils::is_mutable_method;
use crate::compiler::codegen::actions::{FireworkIR, FireworkStatement, FireworkAction, FireworkWidgetField};
use crate::compiler::error::*;

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

    pub spark_id: usize,
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
    pub is_cycle: bool,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            // Нет имён на старте
            variables: HashMap::new(),
            screen_index: 0,
            depth: 0,
            is_cycle: false,
        }
    }

    /// Генерирует рандомный айди экрана
    pub fn screen_index_generate(&mut self) {
        let mut range = rand::thread_rng();
        self.screen_index = range.gen_range(0..=usize::MAX);
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
    
    // Три области видимости которые нужны для реализации лайфтайм детектора
    // Текущая область видимости куда добавляются локальные переменные
    pub scope: Scope,

    // Стэк областей видимости, при вхходе в область видимости делается пуш, при
    // выходе из области видимости pop. Используется для break и continue в менеджере
    // лайфтаймов
    pub old_scope: Vec<Scope>,

    // Дамп область видимости до входа в функцию, нужна для обработки дропа спарков
    // при return
    pub item_scope: Scope,

    // Statement это блок кода от начала до ; фигурных скобок или в некоторых случаях
    // запятой. Нужно точно знать на каком statement мы сейчас. На старте это 1, поэтому
    // итерацию нужно начать с единицы
    statement_index: usize,

    // Флаг который означает был ли использован функциональный виджет layout! в текущем
    // лайаут блоке. Описывать лайаут можно только один раз в лайаут блоке
    descript_layout: bool,

    // Промежуточное представление, строки кода с добавлением семантической метки
    pub ir: FireworkIR,

    // Текущий стейтемент который будет использоваться для пуша в ir
    statement: FireworkStatement,

    // Текущее имя функции, если мы вне функции то None
    function_name: Option<String>,

    // Буферы
    // Буфер который используется для хранения текущего типа в парсинге переменной,
    // если типа не указан то используется значения константы NO_TYPE
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

    // Счётчики чтобы генерировать названия полей глобальной структуры экрана
    widget_counter: usize,
    spark_counter: usize,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            // При старте нет ошибок
            errors: Vec::new(),
            
            output: TokenStream::new(),
            
            // Три области видимости для лайфтайм менеджера
            scope: Scope::new(),
            old_scope: Vec::new(),
            item_scope: Scope::new(),

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
                parent_widget_id: None,
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
        }
    }

    /// Метод для вывода всего что собранно в области видимости
    pub fn log_scope(&self) {
        // println!("{:#?}", self.scope.variables);
    }
   
    /// Метод обёртка над SparkFinder чтобы быстро найти наличие спарка в выражении
    /// используется в коде чтобы проверить явлется ли блок реактивным и получить вектор
    /// спарков который содержит кортеж (имя, айди)
    pub fn get_sparks(&self, expr: &Expr) -> Vec<(String, usize)> {
        let mut found = Vec::new();

        let mut finder = SparkFinderWithId {
            scope: &self.scope,
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
            self.ir.screen_structs.entry(format!("ApplicationUiBlockStruct{}", self.scope.screen_index.to_string()))
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
        self.scope.depth += 1;

        let state = self.reactive_block;
        let condition_has_spark = !sparks.is_empty();
        
        let mut open_statement = self.statement.clone();
        open_statement.string = open_code;
        
        if condition_has_spark && self.reactive_block.is_none() {
            open_statement.action = action;
            open_statement.is_reactive_block = true;
            self.reactive_block = Some((self.statement_index, is_loop));
        } else {
            open_statement.action = FireworkAction::ReactiveElse;
            if !matches!(action, FireworkAction::ReactiveElse) {
                open_statement.action = FireworkAction::DefaultCode;
            }

            open_statement.is_reactive_block = false;
        }
        
        self.ir.statements.push(open_statement);
        self.statement_index += 1;
        
        // let _saved_action = self.statement.action.clone();
        self.statement.action = FireworkAction::DefaultCode;
        
        visit_fn(self);
        
        self.statement.action = FireworkAction::DefaultCode;
        self.statement.string = "}".to_string();
        self.statement_index += 1;

        // Закрывающая фигурная скобка также является частью реактивного блока
        self.statement.is_reactive_block = true;
        
        self.reactive_block = state; 
        
        // Защита от переполнения
        if self.scope.depth > 0 {
            self.scope.depth -= 1;
        }
    }

    /// Систсема для обработки выхода из области видимости, принимает старую область
    /// видимости (scope) после чего делает сравнение с текущей областью видимости,
    /// локальные переменных которые были созданы в этой области видимости нет в старой
    /// области видимости, алгоритм сравнения найдёт отсуствие переменной и сгенерирует
    /// семантическую метку для IR DropSpark, оно означает что нужно вернуть владение
    /// обратно в BSS так как локальная переменная которая арендовала значение из BSS
    /// мертва и чтобы не было UB нужно вернуть значение обратно в BSS память со стэка.
    /// Так как мы делаем push в IR до обработки следующего statement то в IR сначала
    /// будет возврат в этой же области видимости
    ///
    /// Семантика:
    ///  - self.scope это текущая область видимости
    ///  - scope это старая область видимости которая была до входа в текущую область
    ///    видимости
    pub fn update_scope(&mut self, scope: Scope, set_scope: bool) {
        for (name, value) in &self.scope.variables {
            if !scope.variables.contains_key(name) && value.is_spark {
                let mut statement = self.statement.clone();

                statement.string = "".to_string();
                statement.action = FireworkAction::DropSpark {
                    name: name.to_string(),

                    // Айди для определения статического поля экземпляра структуры в
                    // проходе кодогенерации
                    id: value.spark_id,
                };
                self.ir.statements.push(statement);
                self.statement_index += 1;
            }
        }

        if set_scope {
            self.scope = scope;
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
            spark_id: 0, // HARDCODE
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

pub fn prepare_tokens(tokens: Vec<TokenTree>, _id: u64) -> (proc_macro2::TokenStream, Option<proc_macro2::TokenStream>, Option<FireworkIR>) {
    let token_stream: proc_macro2::TokenStream = tokens.into_iter().collect();
    
    let file = match syn::parse2::<File>(token_stream) {
        Ok(file) => file,
        Err(e) => return (proc_macro2::TokenStream::new(), Some(e.to_compile_error()), None),
    };
    
    let mut analyzer = Analyzer::new();
    analyzer.scope.screen_index_generate();
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
