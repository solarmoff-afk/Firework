// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

#[cfg(feature = "trace")]
use tracing::instrument;

#[cfg(feature = "trace")]
use quote::quote;

pub use super::super::*;

use crate::CompileType;

impl CodegenVisitor<'_> {
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(node = %quote!(#item_struct))))]
    pub fn codegen_item_struct(&self, item_struct: &mut ItemStruct) {
        if !matches!(self.flags.compile_type, CompileType::Component) {
            return;
        }

        let struct_name = &item_struct.ident.to_string();

        // Проверка есть ли у структуры именованные поля
        if let Fields::Named(fields_named) = &mut item_struct.fields
            && let Some(fields) = self.ir.component_structs.get(struct_name)
        {
            for (field_name, field_type) in fields {
                let field_name_ident = format_ident!("{}", field_name);
                let type_ident = format_ident!("{}", field_type);

                let new_field: syn::Field = parse_quote! {
                    pub #field_name_ident: core::option::Option<#type_ident>
                };

                fields_named.named.push(new_field);
            }
        }
    }
}
