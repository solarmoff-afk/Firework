use glam::Vec4;
use std::fmt::Debug;

#[derive(Clone, Copy, Debug)]
pub struct Color(pub Vec4);

impl Color {
    pub const RED: Self = Self(Vec4::new(1.0, 0.0, 0.0, 1.0));
    pub const GREEN: Self = Self(Vec4::new(0.0, 1.0, 0.0, 1.0));
    pub const BLUE: Self = Self(Vec4::new(0.0, 0.0, 1.0, 1.0));
    pub const WHITE: Self = Self(Vec4::new(1.0, 1.0, 1.0, 1.0));
    pub const BLACK: Self = Self(Vec4::new(0.0, 0.0, 0.0, 1.0));
}

#[derive(Clone, Debug)]
pub struct Element {
    pub kind: ElementKind,
    pub background_color: Option<Color>,
}

#[derive(Clone, Debug)]
pub enum ElementKind {
    Rect { color: Color },
    Text { content: String, font_size: f32, color: Color },
    Container { children: Vec<Element> },
}

#[macro_export]
macro_rules! container {
    ( $($child:expr),* $(,)? ) => {
        $crate::element::container(vec![$($child),*])
    };
}

#[macro_export]
macro_rules! text {
    ($($arg:tt)+) => {
        $crate::element::text(&format!($($arg)+))
    };
}

#[macro_export]
macro_rules! rect {
    ($color:expr) => {
        $crate::element::rect($color)
    };
}

pub fn text(content: &str) -> Element {
    Element {
        kind: ElementKind::Text {
            content: content.to_string(),
            font_size: 16.0,
            color: Color::BLACK,
        },
        background_color: None,
    }
}

pub fn container(children: Vec<Element>) -> Element {
    Element {
        kind: ElementKind::Container { children },
        background_color: None,
    }
}

pub fn rect(color: Color) -> Element {
    Element {
        kind: ElementKind::Rect { color },
        background_color: None,
    }
}

impl Element {
    pub fn background(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }
}