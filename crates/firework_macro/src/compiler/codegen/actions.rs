// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

#![allow(dead_code)]

use proc_macro2::{Span, TokenStream};
use std::collections::HashMap;

use super::snapshot::{Snapshot, SpanKey};

/// Какой это конкретно реактивный блок
#[derive(Debug, Clone, PartialEq)]
pub enum FireworkReactiveBlock {
    // Условие
    ReactiveIf,

    // Цикл for 
    ReactiveFor,

    // Цикл while 
    ReactiveWhile,

    // Match 
    ReactiveMatch,

    // Эффект, реактивный блок без условия
    Effect,
}

#[derive(Debug, Clone)]
pub enum FireworkAction {
    // Инициализация реактивной переменной (спарка) в области видимости. Первое значение
    // это имя реактивной переменной, а второе это айдишник переменно, третье это
    // тип, четвёртое это выражение внутри spark!( ... )
    InitialSpark {
        name: String,
        id: usize,
        spark_type: String,
        expr_body: String,
        is_mut: bool,
    },

    // Когда спарк выходит из области видимости необходимо вернуть владение обратно в
    // статический экземпляр структуры экрана
    DropSpark {
        name: String,
        id: usize,
    },

    // Использование маркера spark_ref!(global_state), создвёт ссылку (мутабельность зависит
    // от наличия mut) на поле в state! {} сегменте shared!, если использовать в обычном
    // экране, а не в шейдере то должна быть ошибка компиляции
    SparkRef {
        name: String,
        id: usize,
        is_mut: bool,
    },

    // Реактивный блок. Первое значение это вектор с названиями реактивных
    // переменных (спарков) которые используются в блоке и их айди
    ReactiveBlock(FireworkReactiveBlock, Vec<(String, usize)>),

    // Блок else который является частью реактивного условия
    ReactiveElse,

    // Закрывающа фигурная скобка для реактивного блока
    ReactiveBlockTerminator,

    // Обновление значения спарка
    UpdateSpark(String, usize), 

    // Лайаут блок, первое значение это название лайаута, второе значение это нужен
    // ли микрорантайм
    LayoutBlock(String, bool),

    // Виджет, декларативное описание для скина. Виджет это compile-time концепция,
    // скин это runtime концепция
    WidgetBlock(WidgetDescription),

    // Просто код для инлайна
    DefaultCode,

    // Завершение функции экрана
    Terminator,
}

/// Раст команда (statement) записанная анализатором
#[derive(Debug, Clone)]
pub struct FireworkStatement {
    // Семантическая метка которая кратко говорит что делает эта строка, создаёт спарк,
    // обновляет спарк, дропает спарк и так далее
    pub action: FireworkAction,
    
    // Явлется ли это реактивным блоком
    pub is_reactive_block: bool,

    // Текущий индекс (Может быть полезен для дебага, работает через счётчик)
    pub index: usize,

    // Имя экрана к которому относится стейтемент
    pub screen_name: String,

    // Строковое представления собранное из токенов, нужно для инлайна
    pub string: String,

    // К какому виджету принадлежит, это нужно для содержимого замыканий в виджетах чтобы
    // определить какой именно блок кода дёргать зная айди виджета
    pub parent_widget_id: Option<usize>,

    pub reactive_loop: bool,

    pub depth: u16,
    pub screen_index: u128,

    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FireworkIR {
    // Айди элемента в векторе это номер statement
    pub statements: Vec<FireworkStatement>,

    // Карта Спан -> Виртуальный стейтемент
    pub snapshot: Snapshot, 

    // Последний спан который был задан. Используется чтобы разместить 
    pub last_span: Option<SpanKey>,

    // Соотвествие экрана (название функции) и структуры экрана в формате вектора
    // кортежей (Имя поля, тип) для структуры
    pub screen_structs: HashMap<String, Vec<(String, String)>>,

    pub screens: Vec<(
        String,
        String,
        u128, // Id экрана
    )>,

    // Хэшмап для хранения id экрана -> количество спарков
    pub screen_sparks: HashMap<u128, usize>,

    // Хэшмап для хранения id экрана -> количество виджетов
    pub screen_widgets: HashMap<u128, usize>,

    pub shared: SharedData,
}

impl FireworkIR {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
            snapshot: Snapshot::new(), 
            last_span: None,
            screen_structs: HashMap::new(),
            screens: Vec::new(),
            screen_sparks: HashMap::new(),
            screen_widgets: HashMap::new(),
            shared: SharedData::new()
        }
    }
    
    pub fn push(&mut self, stmt: FireworkStatement) {
        let span_key = SpanKey::from_span(stmt.span);
        self.last_span = Some(span_key.clone());
        
        // Теперь просто вставляем, без Vec
        self.snapshot
            .statements
            .entry(span_key.clone())
            .or_insert_with(Vec::new)
            .push(stmt.clone());
        
        self.statements.push(stmt);
        
        if !self.snapshot.order.contains(&span_key) {
            self.snapshot.order.push(span_key);
        }
    }
    
    pub fn set_span(&mut self, span: Span) {
        self.last_span = Some(SpanKey::from_span(span));
    }

    pub fn get_current_span(&self) -> Option<&SpanKey> {
        self.last_span.as_ref()
    }

    pub fn get_current_statements(&self) -> Option<&Vec<FireworkStatement>> {
        if let Some(span_key) = &self.last_span {
            self.snapshot.statements.get(span_key)
        } else {
            None
        }
    }

    pub fn get_current_statements_mut(&mut self) -> Option<&mut Vec<FireworkStatement>> {
        if let Some(span_key) = &self.last_span {
            self.snapshot.statements.get_mut(span_key)
        } else {
            None
        }
    }

    pub fn get_statements_by_span(&self, span: Span) -> Option<&Vec<FireworkStatement>> {
        let key = SpanKey::from_span(span);
        self.snapshot.statements.get(&key)
    }

    pub fn get_statements_by_span_mut(&mut self, span: Span) -> Option<&mut Vec<FireworkStatement>> {
        let key = SpanKey::from_span(span);
        self.snapshot.statements.get_mut(&key)
    }
}

/// Поле виджета в Widget DSL, полем считается отдельная часть общей настройки
/// widget_name! {
///  field: 123, // Поле
/// }
///
/// Необходимо для кодогенерации, так как виджеты это чистая compile-time сущность,
/// в реалтайме есть только скины
#[derive(Debug, Clone)]
pub struct FireworkWidgetField {
    // Какие спарки используются в поле
    pub sparks: Vec<(String, usize)>,
    
    // Полная строка выражения поля
    pub string: String,

    // Выражение (правая часть) поля в формате TokenStream с оригинальными спанами
    pub token_stream: TokenStream,

    // Является ли это замыканием
    pub is_fn: bool,
}

/// Поле разделяемого состояния в shared! {} блоке
/// shared! {
///  state! {
///   my_own: i32 = 10,
///   [name] [type] [init]
///  }
/// }
#[derive(Debug, Clone)]
pub struct FireworkSharedState {
    pub name: String,
    pub spark_type: String,
    pub init: String,
    pub span: Span,
    pub id: usize,
}

/// Структура для хранения информации о shared
#[derive(Debug, Clone)]
pub struct SharedData {
    // Для shared блоков IR содержит вектор состояний которые были объявлены в state! чтобы
    // сгенерировать build функцию для shared блока
    pub state: Vec<FireworkSharedState>,

    // Разделяемое состояние (название) -> Вектор функциональных эффектов
    pub effects: HashMap<String, Vec<String>>,
}

impl SharedData {
    pub fn new() -> Self {
        Self {
            state: Vec::new(),
            effects: HashMap::new(),
        }
    }
}

/// Блок декларативного описания виджета, явлется самым сложным действием. Кодогенератор
/// превращает описание виджета в конструкцию скина через Builder Pattern которые должны
/// реализовать все скины. Каждое декларативное поле кроме исключения (skin) будет
/// сгенерированно как вызов метода в цепочке из скина. Для полей которые имеют спарки
/// внутри будет дополнительная генерация для реактивного обновления
#[derive(Debug, Clone)]
pub struct WidgetDescription {
    /// Тип виджета (Например, rect или text)
    pub widget_type: String,

    /// Карта для полей, String -> FireworkWidgetField, FireworkWidgetField содержит
    /// само поле и спарки которые используются внутри
    pub fields: HashMap<String, FireworkWidgetField>,

    /// Является ли этот виджет функциональным (layout!, component!)
    pub is_functional: bool,
    
    /// Айди виджета
    pub id: usize,

    /// Нужен ли для виджета микрорантайм (динамический список)
    pub has_microruntime: bool,

    /// Скин который использует виджет, его можно задать используя поле skin либо он будет
    /// выбран автоматически
    pub skin: String,

    /// Рендерится ли виджет условно
    pub is_maybe: bool,
}
