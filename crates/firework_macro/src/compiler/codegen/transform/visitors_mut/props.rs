// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use quote::quote;

use super::super::*;

impl CodegenVisitor<'_> {
    /// Метод для генерации сеттеров пропсов компонентов. Он находит среди полей структуры
    /// поля с типом firework_ui::Prop<T> или Prop<T> после чего создаёт публичный метод
    /// сеттер с именем __{имя пропса}
    pub(crate) fn generate_component_setters(
        &self,
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
                let type_str = quote!(#field_type).to_string().replace(" ", "");
                if !type_str.starts_with("firework_ui::Prop<") && !type_str.starts_with("Prop<") {
                    println!("{type_str}");
                    continue;
                }

                let setter_name = format_ident!("__{}", field_name);

                // Сеттер реализует BuilderPattern (цепочку вызовов)
                let setter = parse_quote! {
                    impl #struct_name {
                        pub fn #setter_name(&mut self, value: #field_type) -> &mut Self {
                            self.#field_name = value;
                            self
                        }
                    }
                };

                new_items.push(setter);
            }
        }
    }
}
