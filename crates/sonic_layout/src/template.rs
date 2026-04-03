// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub struct SonicTemplate {
    // Шаблон переменной которую создаёт соник. Шаблоны внутри:
    //  - {name} имя переменной которую создаёт Sonic 
    //  - {type} тип переменной которую создаёт Sonic
    pub layout_variable: String,

    pub set_layout_variable: String,

    pub add_layout_variable: String,

    pub get_layout_variable: String,

    // Измерение виджета, строка должна быть без точки с запятой, её ставит Sonic сам.
    //  - {id} айди виджета, этот айди (usize) был указан при добавлении виджета и должен
    //   использоваться для получения поля
    pub measure_widget: String,

    // Используется чтобы установить позицию объекта, требует точку с запятой в конце
    pub add_position: String,
}

pub(crate) fn replace_placeholders(text: &str, placeholder: &str, replacement: &str) -> String {
    let pattern = format!("{{{}}}", placeholder);
    text.replace(&pattern, replacement)
}
