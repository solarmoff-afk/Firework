pub use crate::element::{Element, ElementKind};

pub fn update_tree(tree: &Element) {
    process_element(tree);
}

fn process_element(element: &Element) {
    let kind = &element.kind;
    
    /*
        Используем match вместо простого сравнения так как
        ElementKid::Container это вариативная структура
        и её нельзя сравнить через if
    */

    match kind {
        ElementKind::Container { children, .. } => {
            for child in children {
                process_element(&child);
            }
        },
        _ => {}
    }
}