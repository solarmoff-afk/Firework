// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use std::collections::HashMap;

use crate::compiler::analyze2::Scope;

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
    },

    // Реактивный блок типа условие. Первое значение это вектор с названиями реактивных
    // переменных (спарков) которые используются в условии
    ReactiveIf(Vec<String>),

    // Реактивный блок типа матч, первое значение это вектор с названиями реактивных
    // переменных (спарков), нужен для match ... { ... };
    ReactiveMatch(Vec<String>),

    // Реактивный цикл for
    ReactiveFor(Vec<String>),

    // Реактивный цикл while
    ReactiveWhile(Vec<String>),

    // Обновление значения спарка
    UpdateSpark(String),

    // Лайаут блок, первое значение это название лайаута, второе значение это нужен
    // ли микрорантайм
    LayoutBlock(String, bool),

    // Блок декларативного описания виджета, явлется самым сложным действием
    // Значения:
    //  1 (String)  - Какой это виджет (text, rect, buttom и так далее)
    //  2 (HashMap) - Соотвествие полей виджета и вектора спарков которые используются
    //  3 (bool)    - Явлется ли это функциональным виджетом (Который не имеет визуального
    //                представления)
    //  4 (usize)   - Айди виджета
    WidgetBlock(String, HashMap<String, FireworkWidgetField>, bool, usize),

    DefaultCode,
}

#[derive(Debug, Clone)]
pub struct FireworkStatement {
    pub action: FireworkAction,
    pub is_reactive_block: bool,
    pub index: usize,
    pub screen_name: String,
    pub string: String,
    pub parent_widget_id: Option<usize>,

    // TODO: Оптимизировать, так как клонировать Scope (HashSet + usize) для каждого
    // statement может быть дорого
    pub scope: Scope,
}

#[derive(Debug, Clone)]
pub struct FireworkIR {
    // Айди элемента в векторе это номер statement
    pub statements: Vec<FireworkStatement>,

    // Соотвествие экрана (название функции) и структуры экрана в формате вектора
    // кортежей (Имя поля, тип) для структуры
    pub screen_structs: HashMap<String, Vec<(String, String)>>,

    pub screens: Vec<(
        String,
        String,
        usize, // Id экрана
    )>,
    
    pub items: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FireworkWidgetField {
    pub sparks: Vec<String>,
    pub string: String,
}