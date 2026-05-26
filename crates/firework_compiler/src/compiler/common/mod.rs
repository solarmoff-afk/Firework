// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub mod find_expr_attrs;
pub mod widget_kind;

use syn::*;

use crate::compiler::common::find_expr_attrs::get_expr_attrs_mut;

pub(crate) fn is_prop(type_str: &str) -> bool {
    let clean = type_str.replace(" ", "");
    clean.starts_with("firework_ui::Prop<") || clean.starts_with("Prop<")
}

/// Поиск атрибутов
pub(crate) fn has_attribute(attrs: &[Attribute], find: &str) -> bool {
    attrs.iter().any(|attr| is_attribute(attr, find))
}

fn is_attribute(attr: &Attribute, find: &str) -> bool {
    attr.path().is_ident(find)
}

pub fn remove_attribute(attrs: &mut Vec<Attribute>, find: &str) {
    attrs.retain(|attr| !attr.path().is_ident(find));
}

pub fn remove_expr_attribute(expr: &mut Expr, find: &str) {
    if let Some(attrs) = get_expr_attrs_mut(expr) {
        attrs.retain(|attr| !attr.path().is_ident(find));
    }
}
