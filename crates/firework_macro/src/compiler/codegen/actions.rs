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

    // Когда спарк выходит из области видимости необходимо вернуть владение обратно в
    // статический экземпляр структуры экрана
    DropSpark {
        name: String,
        id: usize,
    },

    // Реактивный блок типа условие. Первое значение это вектор с названиями реактивных
    // переменных (спарков) которые используются в условии и их id
    ReactiveIf(Vec<(String, usize)>),

    // Реактивный блок типа матч, первое значение это вектор с названиями реактивных
    // переменных (спарков), нужен для match ... { ... };
    ReactiveMatch(Vec<(String, usize)>),

    // Реактивный цикл for
    ReactiveFor(Vec<(String, usize)>),

    // Реактивный цикл while
    ReactiveWhile(Vec<(String, usize)>),

    // Обновление значения спарка
    UpdateSpark(String, usize),

    // Блок else который является частью реактивного условия
    ReactiveElse,

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
   
    // Структуры, трейты и так далее которые определены на верхнем уровне вызова
    // процедурного макроса, нужны для вставки в кодогенерации
    pub items: Vec<String>,
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
    pub sparks: Vec<String>,
    
    // Полная строка поля
    pub string: String,
}
