// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::super::*;

impl CodeBuilder {
    #[tracing::instrument(skip_all, fields(statements = ?statement))]
    pub fn node_widget_block(
        &mut self,
        span: Span,
        struct_name: String,
        final_tokens: &mut TokenStream,
        statement: &FireworkStatement,
    ) -> bool {
        if let FireworkAction::WidgetBlock(description) = &statement.action {
            let instance_ident_upper = format_ident!("{}_INSTANCE", struct_name.to_uppercase());
            let field_ident = format_ident!("widget_object_{}", description.id);

            let skin_path: syn::Type = syn::parse_str(&description.skin)
                .unwrap_or_else(|_| panic!("Invalid skin name: {}", description.skin));

            // При навигации нужно сгенерировать конструкцию виджета на основе скина
            let mut widget_init = quote_spanned! { span=>
                #skin_path::new(1).expect("Failed to create new widget instance")
            };

            let mut widget_reactive = quote! {};

            // Выражение ключа, ключ нужен в динамических списках для оптимизиации
            // обхода в микрорантайме
            let mut key_expr: Option<TokenStream> = None;

            // Обход всех полей
            for (name, field) in &description.fields {
                if name == "key" {
                    key_expr = Some(field.token_stream.clone());
                    continue;
                }

                // Поле с именем skin нужно пропустить, так как оно явлется задающим
                if need_skip_props(name) {
                    continue;
                }

                if field.is_fn && is_event(name) {
                    continue;
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

                if !field.sparks.is_empty() {
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

            // Токен стрим для хранения обновления нужного бита в бит маске (активации
            // бита) чтобы показать что виджет жив
            let mut widget_update_bitmask = TokenStream::new();

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
                let statement = format!(
                    "{};",
                    set_flag(
                        format!("_fwc_widget_bitmask{}", mask).as_str(),
                        normalize_bit_index(local_id),
                    )
                )
                .to_stmt()
                .expect("Widget_block symtax error in bitmask");

                widget_update_bitmask.extend(quote! {
                    #statement
                });
            }

            // Генерация проверки на то, что бит виджета изменился в битовой маске
            let mut condition = String::new();

            // Если description.is_maybe будет None то этот код просто не будет
            // использован, поэтому unwrap_or(0) является нормой, так как 0 хардкод просто
            // не будет использован
            self.generate_check_widget_bit(&mut condition, description.is_maybe.unwrap_or(0));

            let condition_statement = condition
                .to_expr()
                .expect("Widget_block symtax error: Condition statement parse error");

            // Безопасный режим с Mutex
            #[cfg(feature = "safety-multithread")]
            let match_value = quote! {
                #instance_ident_upper.get()
                    .expect("Instance not initialized").lock()
                    .unwrap().#field_ident
            };

            #[cfg(not(feature = "safety-multithread"))]
            let match_value = quote! {
                unsafe {
                    (*::core::ptr::addr_of!(#instance_ident_upper)).#field_ident.as_ref()
                }
            };

            let is_in_loop = description.has_microruntime;
            if is_in_loop {
                // У виджетов в циклах обязан быть ключ, это проверяется анализатором
                let key_token = key_expr.expect("Key field not found");

                #[cfg(feature = "safety-multithread")]
                final_tokens.extend(quote_spanned!(span=>
                    {
                        let mut _fwc_inst = #instance_ident_upper.get()
                            .expect("Instance not initialized").lock().unwrap();

                        // SAFETY: List initialized before
                        let mut _fwc_list_ref = _fwc_inst.#field_ident.as_mut().unwrap();

                        let mut _fwc_wb_1 = match _fwc_list_ref.entry(#key_token) {
                            firework_ui::ListEntry::Occupied(existing) => existing,
                            firework_ui::ListEntry::Vacant(vacant) => vacant.insert(#widget_init),
                        };

                        #widget_reactive
                        #widget_update_bitmask
                    }
                ));

                #[cfg(not(feature = "safety-multithread"))]
                final_tokens.extend(quote_spanned!(span=>
                    {
                        let mut _fwc_list_ref = unsafe {
                            (*::core::ptr::addr_of_mut!(#instance_ident_upper)).#field_ident.as_mut().unwrap()
                        };

                        let mut _fwc_wb_1 = match _fwc_list_ref.entry(#key_token) {
                            firework_ui::ListEntry::Occupied(existing) => existing,
                            firework_ui::ListEntry::Vacant(vacant) => vacant.insert(#widget_init),
                        };

                        #widget_reactive
                        #widget_update_bitmask
                    }
                ));
            } else {
                #[cfg(feature = "safety-multithread")]
                final_tokens.extend(quote_spanned!(span=>
                    match #match_value {
                        Some(ref _fwc_wb_1) => {
                            #widget_reactive
                            #widget_update_bitmask
                        },

                        None => {
                            #instance_ident_upper.get()
                                .expect("Instance not initialized")
                                .lock()
                                .unwrap()
                                .#field_ident = Some(#widget_init);
                            #widget_update_bitmask
                        },
                    };
                ));

                // Обычный режим
                #[cfg(not(feature = "safety-multithread"))]
                final_tokens.extend(quote_spanned!(span=>
                    match #match_value {
                        Some(ref _fwc_wb_1) => {
                            #widget_reactive
                            #widget_update_bitmask
                        },

                        None => {
                            unsafe {
                                (*::core::ptr::addr_of_mut!(#instance_ident_upper)).#field_ident
                                    = Some(#widget_init);
                                #widget_update_bitmask
                            }
                        },
                    };
                ));
            }

            // Финализация
            if description.is_maybe.is_some() {
                self.tokens.push(quote_spanned!(span=>
                    match #match_value {
                        Some(ref _fwc_wb_1) => {
                            if #condition_statement { _fwc_wb_1.visible(true); } else {
                                _fwc_wb_1.visible(false);
                            }
                        },

                        None => {},
                    };
                ));
            }

            return true;
        };

        false
    }
}

/// Метод который определяет нужно ли скипнуть пропс (Этот пропс выполняет функцию инструкций
/// для кодогенератора)
fn need_skip_props(props: &str) -> bool {
    props == "skin" ||    // Для того чтобы изменить отображение виджета
    props == "key" // Для динамических списков
}

fn is_event(props: &str) -> bool {
    props == "on_click"
}
