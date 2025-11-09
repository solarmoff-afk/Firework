use std::collections::HashMap;
use moonwalk::{MoonWalk, ObjectId};
use glam::Vec4;

use crate::moon_bridge::MoonBridge;

pub use crate::element::{Element, ElementKind, ElementId};

pub struct FireTree {
    pub tree: HashMap<ElementId, Vec<ObjectId>>,
}

impl FireTree {
    pub fn new() -> Self {
        Self {
            tree: HashMap::new()
        }
    }
}

pub fn update_tree(root_element: &Element, render_tree: &FireTree, bridge: &MoonBridge) {
    let mut mw = bridge.moonwalk();
    process_element(root_element, render_tree, &mut mw);
}

fn process_element(element: &Element, render_tree: &FireTree, mw: &mut MoonWalk) {
    let object_exists = render_tree.tree.contains_key(&element.id);
    let kind = &element.kind;
    
    /*
        Используем match вместо простого сравнения так как
        ElementKind::Container это вариативная структура
        и её нельзя сравнить через if
    */

    match kind {
        ElementKind::Rect { roundedness } => {
            if !object_exists {
                let id = mw.new_rect();
                mw.config_position(id, glam::Vec2::new(100.0, 100.0));
                mw.config_size(id, glam::Vec2::new(200.0, 150.0));
                mw.config_color(id, Vec4::new(0.0, 1.0, 0.0, 1.0));
            }
        },

        ElementKind::Container { children, .. } => {
            for child in children {
                process_element(&child, render_tree, mw);
            }
        },

        _ => {}
    }
}