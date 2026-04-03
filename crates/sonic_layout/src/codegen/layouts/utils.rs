// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use crate::{replace_placeholders, SonicTemplate};

pub fn generate_add_position(id: usize, x: &str, y: &str, template: &SonicTemplate, output: &mut String) {
    let mut statement = replace_placeholders(
        &template.layout_variable,
        "id",
        &id.to_string(),
    );
    
    statement = replace_placeholders(
        &statement,
        "x",
        x,
    );

    statement = replace_placeholders(
        &statement,
        "y",
        y,
    );
    
    output.push_str(&statement);
}

pub(crate) fn generate_set_variable(name: &str, value: &str, template: &SonicTemplate, output: &mut String) {
    let mut variable_set = replace_placeholders(
        &template.set_layout_variable,
        "name",
        name,
    );
    
    variable_set = replace_placeholders(
        &variable_set,
        "value",
        value,
    );
    
    output.push_str(&variable_set);
}

pub(crate) fn generate_add_variable(name: &str, value: &str, template: &SonicTemplate, output: &mut String) {
    let mut variable_set = replace_placeholders(
        &template.add_layout_variable,
        "name",
        name,
    );
    
    variable_set = replace_placeholders(
        &variable_set,
        "value",
        value,
    );
    
    output.push_str(&variable_set);
}
