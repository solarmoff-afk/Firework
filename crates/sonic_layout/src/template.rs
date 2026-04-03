// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub struct SonicTemplate {
    // Шаблон переменной которую создаёт соник. Шаблоны внутри:
    //  - {name} имя переменной которую создаёт Sonic 
    //  - {type} тип переменной которую создаёт Sonic
    pub layout_variable: String,
}

pub(crate) fn replace_placeholders(text: &str, placeholder: &str, replacement: &str) -> String {
    let pattern = format!("{{{}}}", placeholder);
    text.replace(&pattern, replacement)
}
