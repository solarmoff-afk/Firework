// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use quote::ToTokens;
use quote::quote;

use super::super::*;

use crate::compiler::common::is_prop;

impl CodegenVisitor<'_> {
    /// Метод для генерации сеттеров пропсов компонентов. Он находит среди полей структуры
    /// поля с типом firework_ui::Prop<T> или Prop<T> после чего создаёт публичный метод
    /// сеттер с именем __{имя пропса}
    pub(crate) fn generate_component_setters(
        &mut self,
        item_struct: &mut ItemStruct,
        new_items: &mut Vec<Item>,
    ) {
        let struct_name = &item_struct.ident;

        if let Fields::Named(fields_named) = &item_struct.fields {
            for field in &fields_named.named {
                let field_name = field.ident.as_ref().unwrap();
                let field_type = &field.ty;

                // Генерация сеттера только для пропсов, сеттер нужен для внешних изменений
                // структурных пропсов и отпечатка в битовой маске
                let type_str = quote!(#field_type);
                if !is_prop(&type_str.to_string()) {
                    continue;
                }

                let inner_type = self
                    .get_type_from_prop(field_type)
                    .unwrap_or_else(|| field_type.clone());

                let generics = if let Some(component) =
                    self.ir.component_structs.get_mut(&struct_name.to_string())
                {
                    component.tcomponents.to_token_stream()
                } else {
                    quote! {}
                };

                let setter_name = format_ident!("{}", field_name);
                let setter_reactive_name = format_ident!("__fwc_set_{}", setter_name);

                // Сеттер реализует BuilderPattern (цепочку вызовов)
                let setter = parse_quote! {
                    impl #generics #struct_name #generics {
                        pub fn #setter_name(mut self, value: #inner_type) -> Self {
                            self.#field_name = Some(value);
                            self
                        }

                        pub fn #setter_reactive_name(&mut self, value: #inner_type) -> &mut Self {
                            self.#field_name = Some(value);
                            self
                        }
                    }
                };

                new_items.push(setter);
            }
        }
    }

    /// Разворачивает Prop<T> в T, например, Prop<bool> в Bool. Используется чтобы
    /// сгенерировать функцию установки пропа
    fn get_type_from_prop(&self, field_type: &Type) -> Option<Type> {
        if let Type::Path(TypePath { path, .. }) = field_type {
            for segment in &path.segments {
                if segment.ident == "Prop" {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                            return Some(inner_ty.clone());
                        }
                    }
                }
            }
        }

        None
    }
}
