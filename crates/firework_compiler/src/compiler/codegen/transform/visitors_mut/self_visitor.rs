// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

#[cfg(feature = "trace")]
use tracing::instrument;

#[cfg(feature = "trace")]
use quote::quote;

use proc_macro2::Span;
use syn::visit_mut::{self, VisitMut};

use super::super::*;

pub struct SelfFieldAdder {
    pub fields: Vec<(String, String)>,
}

impl SelfFieldAdder {
    pub fn new(fields: Vec<(String, String)>) -> Self {
        Self { fields }
    }
}

impl VisitMut for SelfFieldAdder {
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(node = %quote!(#expr_struct))))]
    fn visit_expr_struct_mut(&mut self, expr_struct: &mut ExprStruct) {
        if expr_struct.path.is_ident("Self") {
            for (field_name, _) in &self.fields {
                let field_ident = Ident::new(field_name, Span::call_site());

                let field_exists = expr_struct.fields.iter().any(|field_value| {
                    if let Member::Named(ident) = &field_value.member {
                        ident == &field_ident
                    } else {
                        false
                    }
                });

                // Создаём None определение только если такого определения нет в конструкторе
                // или если поле системное и есть в чёрном списке (fields_black_list проверяет)
                if !field_exists && !fields_black_list(format!("{}", field_ident).as_str()) {
                    let field_value: FieldValue = parse_quote! {
                        #field_ident: None
                    };
                    expr_struct.fields.push(field_value);
                }
            }

            expr_struct.fields.push(parse_quote! {
                _fwc__fwc_component: Some(firework_ui::ComponentData::new())
            });
        }

        visit_mut::visit_expr_struct_mut(self, expr_struct);
    }
}

/// Проверка что поле структуры является тем, которое нельзя инициализировать как None в
/// случае отсуствия инициализации в fn new() -> Self
fn fields_black_list(field: &str) -> bool {
    field == "_fwc__fwc_component"
}
