use glam::{Vec2, Vec4};
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

#[derive(Clone, Copy, Debug)]
pub enum Angle {
    Radians(f32),
    Degrees(f32),
}

#[derive(Clone, Debug)]
pub struct Element {
    pub kind: ElementKind,
    pub background_color: Option<Color>,
    pub color: Option<Color>,
    pub position: Option<Vec2>,
    pub size: Option<Vec2>,
    pub rotation: Option<Angle>,
}

impl Default for Element {
    fn default() -> Self {
        Self {
            kind: ElementKind::default(),
            background_color: None,
            color: None,
            position: Some(Vec2::ZERO),
            size: Some(Vec2::ZERO),
            rotation: Some(Angle::Degrees(0.0)),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum ElementKind {
    #[default]
    Empty,
    Rect { roundedness: f32 },
    Text { content: String, font_size: f32 },
    Container { children: Vec<Element> },
}

#[macro_export]
macro_rules! container {
    ( $($child:expr),* $(,)? ) => { $crate::element::container(vec![$($child),*]) };
}

#[macro_export]
macro_rules! text {
    () => {
        $crate::element::text("")
    };
    ($($arg:tt)+) => {
        $crate::element::text(&format!($($arg)+))
    };
}

#[macro_export]
macro_rules! rect {
    () => {
        $crate::element::rect(0.0)
    };

    ($roundedness:expr) => {
        $crate::element::rect($roundedness)
    };
}

pub fn text(content: &str) -> Element {
    Element {
        kind: ElementKind::Text {
            content: content.to_string(),
            font_size: 16.0,
        },
        
        color: Some(Color::BLACK),
        ..Default::default()
    }
}

pub fn container(children: Vec<Element>) -> Element {
    Element {
        kind: ElementKind::Container { children },
        ..Default::default()
    }
}

pub fn rect(roundedness: f32) -> Element {
    Element {
        kind: ElementKind::Rect { roundedness },
        color: Some(Color::BLACK),
        ..Default::default()
    }
}

impl Element {
    pub fn background(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn position<P: Into<Vec2>>(mut self, pos: P) -> Self {
        self.position = Some(pos.into());
        self
    }

    pub fn size<S: Into<Vec2>>(mut self, size: S) -> Self {
        self.size = Some(size.into());
        self
    }

    pub fn rotation(mut self, angle: Angle) -> Self {
        self.rotation = Some(angle);
        self
    }
}