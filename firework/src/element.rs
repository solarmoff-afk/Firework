use glam::{Vec2, Vec4};
use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::layout::{ContentAlignment, Layout};

static NEXT_ELEMENT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ElementId(u64);

impl ElementId {
    pub fn new() -> Self {
        Self(NEXT_ELEMENT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

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
    pub id: ElementId,
    pub kind: ElementKind,
    pub background_color: Option<Color>,
    pub color: Option<Color>,
    pub position: Option<Vec2>,
    pub size: Option<Vec2>,
    pub rotation: Option<Angle>,
    pub roundedness: i8,
    pub layout_weight: f32,
    pub self_alignment: Option<ContentAlignment>,
    pub z_index: i32,
    pub dirty: bool,
}

impl Default for Element {
    fn default() -> Self {
        Self {
            id: ElementId::new(),
            kind: ElementKind::default(),
            background_color: None,
            color: None,
            position: Some(Vec2::ZERO),
            size: Some(Vec2::new(100.0, 100.0)),
            rotation: Some(Angle::Degrees(0.0)),
            roundedness: 0,
            layout_weight: 0.0,
            self_alignment: None,
            z_index: 0,
            dirty: true,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum ElementKind {
    #[default]
    Empty,
    Rect { roundedness: i8 },
    Text { content: String, font_size: f32 },
    Container {
        children: Vec<Element>,
        layout: Layout,
    },
}

#[macro_export]
macro_rules! container {
    ( layout: $layout:expr, $($child:expr),* $(,)? ) => {
        $crate::element::container($layout, vec![$($child),*])
    };
    
    ( $($child:expr),* $(,)? ) => {
        $crate::element::container(Default::default(), vec![$($child),*])
    };
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
        $crate::element::rect(0)
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

pub fn container(layout: Layout, children: Vec<Element>) -> Element {
    Element {
        kind: ElementKind::Container { children, layout },
        ..Default::default()
    }
}

pub fn rect(roundedness: i8) -> Element {
    Element {
        kind: ElementKind::Rect { roundedness },
        color: Some(Color::BLACK),
        roundedness: roundedness,
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

    pub fn layout_weight(mut self, weight: f32) -> Self {
        self.layout_weight = weight;
        self
    }
    
    pub fn self_alignment(mut self, align: ContentAlignment) -> Self {
        self.self_alignment = Some(align);
        self
    }
    
    pub fn z_index(mut self, index: i32) -> Self {
        self.z_index = index;
        self
    }
}

#[macro_export]
macro_rules! stack {
    ( $($child:expr),* $(,)? ) => {
        $crate::container![
            layout: $crate::layout::Layout::Unwa($crate::layout::Unwa {
                layout: $crate::layout::LayoutType::Stack,
                ..Default::default()
            }),
            $($child),*
        ]
    };
}

#[macro_export]
macro_rules! row {
    ( $($child:expr),* $(,)? ) => {
        $crate::container![
            layout: $crate::layout::Layout::Unwa($crate::layout::Unwa {
                layout: $crate::layout::LayoutType::Row,
                ..Default::default()
            }),
            $($child),*
        ]
    };
}

#[macro_export]
macro_rules! column {
    ( $($child:expr),* $(,)? ) => {
        $crate::container![
            layout: $crate::layout::Layout::Unwa($crate::layout::Unwa {
                layout: $crate::layout::LayoutType::Column,
                ..Default::default()
            }),
            $($child),*
        ]
    };
}