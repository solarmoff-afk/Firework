// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

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

                if !field_exists {
                    let field_value: FieldValue = parse_quote! {
                        #field_ident: None
                    };
                    expr_struct.fields.push(field_value);
                }
            }
        }

        visit_mut::visit_expr_struct_mut(self, expr_struct);
    }
}
