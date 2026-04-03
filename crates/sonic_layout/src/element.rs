// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

#[derive(Debug, Clone)]
pub enum WidgetType {
    Fixed,
    Fill,
}

#[derive(Debug, Clone)]
pub enum ContainerType {
    Vertical,
    Horizontal,
    Stack,
    Absolute,
    SpaceBetweenVertical,
    SpaceBetweenHorizontal,
}

#[derive(Debug, Clone)]
pub enum Element {
    Widget(WidgetType),

    MaybeWidget(WidgetType),

    Container(ContainerType, Vec<Element>),

    MaybeContainer(ContainerType, Vec<Element>),

    None,
}

#[derive(Debug, Clone)]
pub(crate) struct StackFrame {
    pub container_type: ContainerType,
    pub children: Vec<Element>,
}
