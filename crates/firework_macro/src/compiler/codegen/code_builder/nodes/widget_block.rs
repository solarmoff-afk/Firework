// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::super::*;

impl CodeBuilder { 
    pub fn node_widget_block(
        &self, span: Span, struct_name: String, final_tokens: &mut TokenStream,
        statement: &FireworkStatement, 
    ) {
        match &statement.action {
            FireworkAction::WidgetBlock(
                _widget_type, fields, _is_functional, id, _has_microruntime, skin,
            ) => {
                let instance_ident_upper = format_ident!("{}_INSTANCE", struct_name.to_uppercase());
                let field_ident = format_ident!("widget_object_{}", id);

                let skin_path: syn::Path = syn::parse_str(skin)
                    .expect(format!("Invalid skin name: {}", skin).as_str());

                // При навигации нужно сгенерировать конструкцию виджета на основе скина
                let mut widget_init = quote_spanned! { span=>
                    #skin_path::new(1).unwrap()
                };

                let mut widget_reactive = quote! {};

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

                    if field.sparks.len() > 0 {
                        let mut condition = String::new();

                        // Генерация условия на то, что хотя-бы одна зависимость в снапшотах
                        // битовых масках изменилась
                        for (_, id) in field.sparks.iter() {
                            self.generate_check_spark_bit(&mut condition, *id);
                            condition.push_str(" ||");
                        }
                        
                        // Для упрощения кодогенерации сюда добавляется false для условия
                        condition.push_str(" false ");
                        let condition_statement = condition.to_expr().unwrap();

                        widget_reactive.extend(quote! {
                            if #condition_statement {
                                _fwc_wb_1.#method_ident(#field_value);
                            }
                        });
                    }
                }

                // Безопасный режим с Mutex
                #[cfg(feature = "safety-multithread")]
                final_tokens.extend(quote_spanned!(span=>
                    match #instance_ident_upper.get()
                        .expect("Instance not initialized").lock().unwrap().#field_ident
                    {
                        Some(ref _fwc_wb_1) => {
                            #widget_reactive
                        },

                        None => {
                            #instance_ident_upper.get()
                                .expect("Instance not initialized")
                                .lock()
                                .unwrap()
                                .#field_ident = Some(#widget_init);
                        },
                    };
                ));

                // Обычный режим
                #[cfg(not(feature = "safety-multithread"))]
                final_tokens.extend(quote_spanned!(span=>
                    match unsafe {
                        (*::core::ptr::addr_of!(#instance_ident_upper)).#field_ident
                    } {
                        Some(ref _fwc_wb_1) => {
                            #widget_reactive
                        },
                        
                        None => {
                            unsafe {
                                (*::core::ptr::addr_of_mut!(#instance_ident_upper)).#field_ident
                                    = Some(#widget_init);
                            }
                        },
                    };
                ));
            },

            _ => {},
        };
    }
}
