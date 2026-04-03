// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use crate::{Element, ElementKind, SonicTemplate, replace_placeholders};

/// Эта функция генерирует все необходимые переменные для работы кода который генерирует 
/// Sonic, принимает ссылку на дерево элементов, шаблон и возвращает строку с сгенерированным
/// кодом
pub(crate) fn generate_struct(element: &Element, template: &SonicTemplate) -> String {
    let mut output = "".to_string();
    process_element(element, template, &mut output);

    output
}

fn process_element(element: &Element, template: &SonicTemplate, output: &mut String) {
    match &element.kind {
        ElementKind::Container(container_type, children) => {
            let mut output_buffer = "".to_string();

            output_buffer = replace_placeholders(
                &template.layout_variable,
                "name",
                format!("{}_cursor", element.id).as_str(),
            );

            output_buffer = replace_placeholders(
                &output_buffer,
                "type",
                "f32",
            );

            output.push_str(output_buffer.as_str());

            for child in children {
                process_element(&child, template, output);
            }
        }

        _ => {}
    }
}
