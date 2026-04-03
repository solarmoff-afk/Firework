// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use crate::{Element, ElementKind, SonicTemplate, replace_placeholders, ContainerType};

pub(crate) fn generate_layout(element: &Element, template: &SonicTemplate) -> String {
    let mut output = "".to_string();
    process_element(element, template, &mut output);

    output
}

fn process_element(element: &Element, template: &SonicTemplate, output: &mut String) {
    match &element.kind {
        ElementKind::Container(container_type, children) => {
            match container_type {
                ContainerType::Vertical => {
                    println!("Vertical");
                },

                _ => todo!(),
            };

            for child in children {
                process_element(&child, template, output);
            }
        },
        
        _ => {},
    }
}
