#[derive(Clone, Copy, Debug, Default)]
pub enum FlexDirection {
    #[default]
    Row,
    Column,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum JustifyContent {
    #[default]
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum AlignItems {
    #[default]
    Stretch,
    Start,
    End,
    Center,
}

#[derive(Clone, Debug)]
pub struct Flex {
    pub direction: FlexDirection,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub gap: f32,
}

impl Default for Flex {
    fn default() -> Self {
        Self {
            direction: FlexDirection::Column,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Stretch,
            gap: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Layout {
    Flex(Flex),
}

impl Default for Layout {
    fn default() -> Self {
        Layout::Flex(Flex::default())
    }
}