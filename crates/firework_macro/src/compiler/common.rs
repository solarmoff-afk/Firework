// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub(crate) fn is_prop(type_str: &String) -> bool {
    let clean = type_str.replace(" ", "");
    clean.starts_with("firework_ui::Prop<") || clean.starts_with("Prop<")
}
