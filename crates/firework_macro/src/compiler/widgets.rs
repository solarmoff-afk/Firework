// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub fn is_widget_macro(path: &syn::Path) -> bool {
    path.is_ident("vertical") || path.is_ident("horizontal") ||
    path.is_ident("stack")
}
