// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use syn::Expr;
use syn::visit_mut::VisitMut;

use super::super::*;

use crate::compiler::CodegenVisitor;

impl CodeBuilder {
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(span = ?span)))]
    pub fn node_widget_block(
        &mut self,
        span: Span,
        struct_name: String,
        final_tokens: &mut TokenStream,
        statement: &FireworkStatement,
        visitor: &mut CodegenVisitor,
    ) -> bool {
        if let FireworkAction::WidgetBlock(description) = &statement.action {
            // Не кэшируется так как здесь профилирование показывает что расходы HashMap
            // выше чем экономия, без кэширование 780 микросекунд, с нмм ~970
            let instance_ident_upper = format_ident!("{}_INSTANCE", struct_name.to_uppercase());
            let field_ident = format_ident!("_fwc_widget_object_{}", description.id);

            let mut skin_path = self.cache.cache_skin_path(&description.skin);

            // Если это декларация компонента (функциональный виджет component!), то
            // необходимо записать в skin_path название структуры из поля target
            let is_component_declaration = if description.widget_type == "component" {
                let mut raw_component_path = String::new();
                for (name, field) in &description.fields {
                    if name == "target" {
                        raw_component_path = field.string.clone();
                    }
                }

                skin_path = self.cache.cache_skin_path(&raw_component_path);
                true
            } else {
                false
            };

            // При навигации нужно сгенерировать конструкцию виджета на основе скина
            let constructor = match description.widget_type.as_str() {
                "component" => quote_spanned! { span=> new() },
                _ => quote_spanned! { span=> new(1) },
            };

            let mut widget_init = quote_spanned! { span=>
                #skin_path::#constructor.expect("Failed to create widget instance")
            };

            let mut widget_reactive = quote! {};

            // Выражение ключа, ключ нужен в динамических списках для оптимизиации
            // обхода в микрорантайме
            let mut key_expr: Option<TokenStream> = None;

            // Обход всех полей
            for (name, field) in &description.fields {
                // Поле в формате TokenStream для сохранения спанов при ошибках
                let field_value = &field.token_stream;

                if name == "key" {
                    key_expr = Some(field.token_stream.clone());
                    continue;
                }

                // Target у функционального виджета компонента это функциональное поле
                // которое не нужно в аргументах
                if is_component_declaration && name == "target" {
                    continue;
                }

                // Поле с именем skin нужно пропустить, так как оно явлется задающим
                if need_skip_props(name) {
                    continue;
                }

                if field.is_fn && is_event(name) {
                    // issue #4
                    {
                        let mut closure_expr: Expr = match syn::parse2(field_value.clone()) {
                            Ok(expr) => expr,
                            Err(_) => {
                                // Парсинг не может быть провален так как в случае синтаксической
                                // ошибки выполнение не дошло бы сюда
                                continue;
                            }
                        };

                        // Трнасформация замыкания
                        visitor.visit_expr_mut(&mut closure_expr);

                        widget_reactive.extend(quote_spanned! (span => {
                            if firework_ui::tiny_matches!(_fwc_event, firework_ui::LifeCycle::Event) {
                                if let firework_ui::CurrentEvent::Touch {
                                    hit_object_id: Some(id), phase, ..
                                } = firework_ui::take_current_event() {
                                    if id == _fwc_wb_1.__id() && firework_ui::tiny_matches!(phase, firework_ui::AdapterClickPhase::Ended) {
                                        let mut _fwc_cl = #closure_expr;
                                        _fwc_cl();
                                    }
                                }
                            }
                        }));

                        continue;
                    }
                }

                // Название метода берётся из названия поля
                let method_ident = format_ident!("{}", name);

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
                    let mut condition = Vec::new();

                    // Генерация условия на то, что хотя-бы одна зависимость в снапшотах
                    // битовых масках изменилась
                    for (_, id) in field.sparks.iter() {
                        condition.push(check_flag_tokens(
                            &get_mask_name(*id),
                            normalize_bit_index(*id),
                        ));
                    }

                    widget_reactive.extend(quote! {
                        if #( #condition )||* {
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
            let condition_statement = if let Some(local_id) = description.is_maybe {
                let mask_id = get_spark_mask(local_id);
                let bit = normalize_bit_index(local_id);
                let mask_name = self.cache.cache_widget_bitmask(mask_id);

                widget_update_bitmask.extend(quote! {
                    #mask_name.set(#mask_name.get() | (1 << #bit));
                });

                quote! { (#mask_name.get() & (1 << #bit)) != 0 }
            } else {
                quote! { true }
            };

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
                            // widget_update_bitmask всегда должен стоять выше widget_reactive
                            // это нужно чтобы при изменении состояния в on_click или другом
                            // ивенте который обрабатывается в widget_reactive наборе токенов
                            // изменения в виджетных битмасках срабатывали. Дело в том, что
                            // в widget_update_bitmask битмаске в которой бит виджета и самому
                            // биту виджета всегда даётся 1. Но если в on_click написать строку
                            // с изменением состояния связанного с виджетом то в
                            // widget_reactive бит может стать 0. Благодаря такому порядку
                            // дефольная единица на бит виджета из widget_update_bitmask
                            // не будет вляить на ивенты из widget_reactive
                            #widget_update_bitmask
                            #widget_reactive
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
                            #widget_update_bitmask
                            #widget_reactive
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
                            if #condition_statement {
                                _fwc_wb_1.visible(true);
                            } else {
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
