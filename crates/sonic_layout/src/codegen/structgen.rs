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
            // Курсор
            generate_layout_variable(
                format!("{}_cursor", element.id).as_str(),
                "f32",
                template,
                output,
            );

            // Размер всех детей
            generate_layout_variable(
                format!("{}_total_size", element.id).as_str(),
                "f32",
                template,
                output,
            );

            // Количество fill элементов
            generate_layout_variable(
                format!("{}_fill_count", element.id).as_str(),
                "f32",
                template,
                output,
            );

            for child in children {
                process_element(&child, template, output);
            }
        }

        _ => {}
    }
}

fn generate_layout_variable(name: &str, var_type: &str, template: &SonicTemplate, output: &mut String) {
    let mut var_decl = replace_placeholders(
        &template.layout_variable,
        "name",
        name,
    );
    
    var_decl = replace_placeholders(
        &var_decl,
        "type",
        var_type,
    );
    
    output.push_str(&var_decl);
}
