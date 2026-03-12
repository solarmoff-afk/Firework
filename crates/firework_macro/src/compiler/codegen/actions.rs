// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

#[derive(Debug, Clone, Copy)]
pub enum WidgetType {
    Rect,
    Text,
}

#[derive(Debug, Clone, Copy)]
pub enum FireworkAction {
    // Этот statement не нуждается в обработке
    DefaultCode,

    // Создание виджета который подписан на спарк
    CreateWidget(WidgetType),

    // Создание виджета без спарка
    CreateWidgetWithoutSpark(WidgetType),

    // Создание динамического виджета который требует микрорантайм 
    CreateDynamicWidget(WidgetType),

    // Создание динамического виджета без спарка
    CreateDynamicWidgetWithoutSpark(WidgetType),

    // Цикл for который зависит от спарка
    ForSpark,

    // Цикл while который зависит от смарка
    WhileSpark,

    // Условие которое зависит от спарка 
    IfSpark,

    // Матч который зависит от спарка
    MatchSpark,

    // Инициализация спарка
    InitialSpark,

    // Обновление спарка (spark1 = 5, spark1 += 1, spark1.push(...))
    SparkUpdate,
}

#[derive(Debug, Clone, Copy)]
pub struct FireworkStatement {
    pub action: FireworkAction,
    pub index: usize,
}
