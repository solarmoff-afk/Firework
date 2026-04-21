// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::super::*;

impl CodeBuilder { 
    pub fn node_widget_block(
        &self, span: Span, _struct_name: String, final_tokens: &mut TokenStream,
        statement: &FireworkStatement, 
    ) {
        match &statement.action {
            FireworkAction::WidgetBlock(
                _widget_type, fields, _is_functional, _id, _has_microruntime, skin,
            ) => {
                let skin_path: syn::Path = syn::parse_str(skin)
                    .expect(format!("Invalid skin name: {}", skin).as_str());

                // При навигации нужно сгенерировать конструкцию виджета на основе скина
                let mut widget_init = quote_spanned! { span=>
                    #skin_path::new(1).unwrap()
                };

                // Обход всех полей
                for (name, field) in fields {
                    // Поле с именем skin нужно пропустить, так как оно явлется задающим
                    if name == "skin" {
                        return;
                    }

                    // Название метода берётся из названия поля
                    let method_ident = format_ident!("{}", name);

                    // Поле в формате TokenStream для сохранения спанов при ошибках
                    let field_value = &field.token_stream;

                    // Генерируется установка значения по билдер паттерну. Через точку
                    // вызывается метод, имя метода должено соотвестовать названию
                    // поля. Внутрь метода пробрасывается само значение
                    //
                    // rect! {
                    //  position: (10, 10),
                    // }
                    //
                    // Превращается в
                    // // Структура скина и айди лайаута в аргументах
                    // [SKIN]::new(1).unwrap()
                    //  .position((10, 10)) // Имя поля становится вызовом метода
                    //                       // а вторая часть выражения становится
                    //                       // аргументов этого метода
                    widget_init.extend(quote! {
                        .#method_ident(#field_value)
                    });
                }

                // Сборка условия на инициализацию виджета
                let condition = String::from(CHECK_NAVIGATE); 

                // SAFETY: CHECK_NAVIGATE всегда является валидным раст кодом
                let condition_statement = condition.to_expr().unwrap();

                println!("Output: {}", widget_init);

                final_tokens.extend(quote_spanned!(span=>
                    if #condition_statement {
                        #widget_init; 
                    }
                ));
            },

            _ => {},
        };
    }
}
