// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

#[derive(Debug, Clone, Copy)]
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
pub enum ElementKind {
    Widget(WidgetType, usize),

    MaybeWidget(WidgetType, usize),

    Container(ContainerType, Vec<Element>),

    MaybeContainer(ContainerType, Vec<Element>),

    None,
}

#[derive(Debug, Clone)]
pub struct Element {
    pub kind: ElementKind,
    pub id: usize,
}

impl Element {
    pub fn new() -> Self {
        Self {
            kind: ElementKind::None,
            id: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct StackFrame {
    pub container_type: ContainerType,
    pub children: Vec<Element>,
}
