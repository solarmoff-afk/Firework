// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use proc_macro2::TokenStream;

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
    pub fields: Vec<(String, FireworkWidgetField)>,

    /// Является ли этот виджет функциональным (layout!, component!)
    pub is_functional: bool,

    /// Айди виджета
    pub id: usize,

    /// Нужен ли для виджета микрорантайм (динамический список)
    pub has_microruntime: bool,

    /// Скин который использует виджет, его можно задать используя поле skin либо он будет
    /// выбран автоматически
    pub skin: String,

    /// Рендерится ли виджет условно (Some) и если да то какой у него индекс в битовыъ масок
    /// на основе maybe_widgets_counter
    pub is_maybe: Option<usize>,

    pub special: SpecialWidget,
}

#[derive(Debug, Clone)]
pub enum SpecialWidget {
    None,
    Timer(TimerWidget),
}

/// Таймер это функциональный виджет, который выполняет какое-то действие спустя указанное
/// количество времени. Таймер имеет два режима работы
///  - Через какое-то время записывает значение в спарк
///  - Через какое-то время выполняет on_timeout замыкание (работает только если сам таймер
///    не в замыкании)
///
/// Оба режима можно использовать вместе
#[derive(Debug, Clone)]
pub struct TimerWidget {
    pub write: bool,
    pub event: bool,
}
