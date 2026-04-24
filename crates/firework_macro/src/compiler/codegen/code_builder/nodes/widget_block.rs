// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::super::*;

impl CodeBuilder { 
    pub fn node_widget_block(
        &self, span: Span, struct_name: String, final_tokens: &mut TokenStream,
        statement: &FireworkStatement, 
    ) {
        match &statement.action {
            FireworkAction::WidgetBlock(description) => {
                let instance_ident_upper = format_ident!("{}_INSTANCE", struct_name.to_uppercase());
                let field_ident = format_ident!("widget_object_{}", description.id);

                let skin_path: syn::Path = syn::parse_str(&description.skin)
                    .expect(format!("Invalid skin name: {}", description.skin).as_str());

                // При навигации нужно сгенерировать конструкцию виджета на основе скина
                let mut widget_init = quote_spanned! { span=>
                    #skin_path::new(1).unwrap()
                };

                let mut widget_reactive = quote! {};

                // Обход всех полей
                for (name, field) in &description.fields {
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

                    // Если отрисовка виджета является условной (он создан внутри условия
                    // либо match) то он нужно в Some ветке делать его бит в битовой маске
                    // активным. Битовая маска создаётся в самом начале функции и нулевая,
                    // это означает что все условные виджеты будут невидмыми, после чего
                    // во всех блоках декларации виджета будет установка нужного бита
                    // в маске. Тем самым условные виджеты для которых не сработает условие
                    // останутся нулями в битовой маске и будут скрыты. Тем самым условный
                    // рендеринг будет работать для любых условий
                    if let Some(local_id) = description.is_maybe {
                        let mask = get_spark_mask(local_id);
                        let statement = format!("{};", set_flag(
                            format!("_fwc_widget_bitmask{}", mask).as_str(), 
                            normalize_bit_index(local_id),
                        )).to_stmt().unwrap();

                        widget_reactive.extend(quote! {
                            #statement
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
