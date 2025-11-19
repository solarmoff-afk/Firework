use std::collections::HashMap;
use moonwalk::{MoonWalk, ObjectId};
use glam::{Vec2};

use crate::moon_bridge::MoonBridge;

pub use crate::element::{Element, ElementKind, ElementId, Color};

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
        // ElementKind::Rect { roundedness } => {
        ElementKind::Rect { .. } => {
            if !object_exists {
                let id = mw.new_rect();
                mw.config_position(id, element.position.unwrap_or(Vec2::new(0.0, 0.0)));
                mw.config_size(id, element.size.unwrap_or(Vec2::new(100.0, 100.0)));
                
                let color_vec4 = element.color.unwrap_or(Color::WHITE).0;

                mw.config_color(id, color_vec4);
                
                // Код для кривой безье
                // let bezier_id = mw.new_bezier();

                // let points = vec![
                //     glam::Vec2::new(100.0, 300.0),
                //     glam::Vec2::new(200.0, 100.0),
                //     glam::Vec2::new(400.0, 500.0),
                //     glam::Vec2::new(500.0, 300.0),
                // ];

                // mw.set_bezier_points(bezier_id, points);

                // mw.config_color(bezier_id, glam::Vec4::new(1.0, 0.5, 0.0, 1.0));
                // mw.config_bezier_thickness(bezier_id, 5.0);
                // mw.config_bezier_smooth(bezier_id, 1.5);
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