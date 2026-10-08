// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use syn::Expr;
use syn::visit_mut::VisitMut;

use super::super::*;

use crate::CompileType;
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
            let is_component = matches!(visitor.flags.compile_type, CompileType::Component);

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
            let is_component_widget = description.widget_type == "component";
            let mut widget_init = match description.widget_type.as_str() {
                "component" => quote_spanned! { span=> #skin_path::new() },
                _ => quote_spanned! { span=>
                    #skin_path::new(1).expect("Failed to create widget instance")
                },
            };

            let mut widget_reactive = quote! {};

            // Выражение ключа, ключ нужен в динамических списках для оптимизиации
            // обхода в микрорантайме
            let mut key_expr: Option<TokenStream> = None;
            let mut widget_has_event = false;

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
                    widget_has_event = true;

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

                    // Пометить _fwc_z_dirty чтобы пересчитать z координаты виджетов можно
                    // только при изменении поля с именем z или если это component. Поле
                    // z само меняет z координату, из-за чего нужно подогнать все виджеты
                    // экрана или компонента, а также если это изменение поля декларации
                    // компонента, то нужно также пересчитать z, так как мы не знаем, что
                    // находится в компоненте и есть ли там что-то, меняющее z порядок
                    // - TODO: Теоритечски можно сделать более быстрее и получать от
                    // компонента его z dirty флаг, это нужно посмотреть как альтернативу
                    let mark_z_dirty = if name == "z" || is_component_widget {
                        quote! { _fwc_z_dirty = true; }
                    } else {
                        quote! {}
                    };

                    let method_name = if is_component_widget {
                        format_ident!("__fwc_set_{}", method_ident)
                    } else {
                        method_ident
                    };

                    widget_reactive.extend(quote! {
                        if #( #condition )||* {
                            _fwc_wb_1.#method_name(#field_value);
                            #mark_z_dirty
                        }
                    });
                }
            }

            if is_component {
                widget_init.extend(quote! {
                    // ComponentData существует всегда, а без unwrap/expect нельзя сделать
                    // это в рамках Builder pattern
                    .__set_vcanvas(self._fwc__fwc_component.as_ref().expect("IE:14").substrate)
                });
            }

            let post_widget = if is_component_widget {
                // Здесь мы добавляем в widget_reactive создание _fwc_wb_2 чтобы обойти
                // бороу чекер, так как он не даст нам сделать это в post_widget, так как
                // до post_widget мы заимствуем _fwc_wb_1
                widget_reactive.extend(quote! { let _fwc_wb_2 = _fwc_wb_1.__is_dirty(); });

                quote! {
                    if _fwc_wb_2 {
                        let _fwc_context = firework_ui::BuildContext {
                            depth: _fwc_depth,
                            cycle: firework_ui::LifeCycle::Reactive,
                        };

                        _fwc_component_instance.flash(_fwc_context);
                    }

                    if firework_ui::tiny_matches!(_fwc_event, firework_ui::LifeCycle::Event) {
                        let _fwc_context = firework_ui::BuildContext {
                            depth: _fwc_depth,
                            cycle: _fwc_event,
                        };

                        _fwc_component_instance.__fwc_event(_fwc_context);
                    }
                }
            } else {
                quote! {}
            };

            if widget_has_event {
                // Если это компонент у которое есть event в декларации (верхняя
                // проверка на is_event), то происходит вызов __set_event метода
                // который есть только у компонентов и добавляет hit группу подложке
                // компонента
                if is_component_widget {
                    widget_init.extend(quote! {
                        .__set_event()
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

            #[cfg(not(feature = "safety"))]
            let _match_value = quote! {
                unsafe {
                    (*::core::ptr::addr_of_mut!(#instance_ident_upper)).#field_ident.as_mut()
                }
            };

            // Распределитель (distributor) это фильтр который фильтрует фазы циклы перед
            // передачей дочернему компоненту
            let distributor = if is_component_declaration {
                quote_spanned!(span=>
                    {
                        if matches!(
                            _fwc_event,
                            firework_ui::LifeCycle::Build | firework_ui::LifeCycle::Navigate
                        ) {
                            let _fwc_context = firework_ui::BuildContext {
                                depth: _fwc_depth,
                                cycle: _fwc_event,
                            };

                            _fwc_component_instance.flash(_fwc_context);
                        }
                    }
                )
            } else {
                quote::quote!()
            };

            let is_in_loop = description.has_microruntime;
            if is_in_loop {
                // У виджетов в циклах обязан быть ключ, это проверяется анализатором
                let key_token = key_expr.expect("Key field not found");

                let inner_tokens = quote_spanned!(span=>
                    // При любом изменении списка необходимо пересчитать z, так как это меняет
                    // z всех других элементов экрана или компонента
                    _fwc_z_dirty = true;

                    let mut _fwc_wb_1 = match _fwc_list_ref.entry(#key_token) {
                        firework_ui::ListEntry::Occupied(existing) => existing,
                        firework_ui::ListEntry::Vacant(vacant) => vacant.insert(#widget_init),
                    };

                    {
                        let _fwc_component_instance = _fwc_wb_1;
                        #distributor
                        _fwc_wb_1 = _fwc_component_instance;
                    }

                    #widget_reactive
                    #widget_update_bitmask
                );

                #[cfg(feature = "safety")]
                {
                    if is_component {
                        final_tokens.extend(quote_spanned!(span=>
                            {
                                let mut _fwc_list_ref = self.#field_ident.as_mut().unwrap();
                                #inner_tokens
                            }
                        ));
                    } else {
                        final_tokens.extend(quote_spanned!(span=>
                            #instance_ident_upper.with(|inst| {
                                let mut _fwc_inst = inst.borrow_mut();
                                let mut _fwc_list_ref = _fwc_inst.#field_ident.as_mut().unwrap();
                                #inner_tokens
                            });
                        ));
                    }
                }

                #[cfg(not(feature = "safety"))]
                {
                    if is_component {
                        final_tokens.extend(quote_spanned!(span=>
                            {
                                let mut _fwc_list_ref = self.#field_ident.as_mut().unwrap();
                                #inner_tokens
                            }
                        ));
                    } else {
                        final_tokens.extend(quote_spanned!(span=>
                            {
                                let mut _fwc_list_ref = unsafe {
                                    (*::core::ptr::addr_of_mut!(#instance_ident_upper)).#field_ident.as_mut().unwrap()
                                };
                                #inner_tokens
                            }
                        ));
                    }
                }
            } else {
                #[cfg(feature = "safety")]
                {
                    let some_safe = quote_spanned!(span =>
                        Some(ref mut _fwc_wb_1) => {
                            {
                                let _fwc_component_instance: &mut _ = &mut *_fwc_wb_1;
                                #distributor
                            }

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

                            // Для случаев, когда ссылка нужна для флэша
                            {
                                let _fwc_component_instance: &mut _ = &mut *_fwc_wb_1;
                                #post_widget
                            }
                        },
                    );
                    if !is_component {
                        final_tokens.extend(quote_spanned!(span=>
                            #instance_ident_upper.with(|inst| {
                                let mut _fwc_inst = inst.borrow_mut();
                                match _fwc_inst.#field_ident.as_mut() {
                                    #some_safe
                                    None => {
                                        _fwc_inst.#field_ident = Some(#widget_init);
                                        {
                                            let _fwc_component_instance = _fwc_inst.#field_ident.as_mut().unwrap();
                                            #distributor
                                        }
                                        #widget_update_bitmask
                                    },
                                };
                            });
                        ));
                    } else {
                        final_tokens.extend(quote_spanned!(span=>
                            match self.#field_ident.as_mut() {
                                #some_safe
                                None => {
                                    self.#field_ident = Some(#widget_init);
                                    {
                                        let _fwc_component_instance = self.#field_ident.as_mut().unwrap();
                                        #distributor
                                    }
                                    #widget_update_bitmask
                                },
                            };
                        ));
                    }
                }

                // Обычный режим
                #[cfg(not(feature = "safety"))]
                {
                    let some_unsafe = quote_spanned!(span =>
                        Some(ref mut _fwc_wb_1) => {
                            {
                                let _fwc_component_instance = &mut *_fwc_wb_1;
                                #distributor
                            }

                            #widget_update_bitmask
                            #widget_reactive

                            {
                                let _fwc_component_instance = &mut *_fwc_wb_1;
                                #post_widget
                            }
                        },
                    );

                    if !is_component {
                        final_tokens.extend(quote_spanned!(span=>
                            match #_match_value {
                                #some_unsafe

                                None => {
                                    unsafe {
                                        let slot =
                                            &mut (*::core::ptr::addr_of_mut!(#instance_ident_upper)).#field_ident;

                                        *slot = Some(#widget_init);

                                        {
                                            let _fwc_component_instance = slot.as_mut().unwrap_unchecked();
                                            #distributor
                                        }
                                    }

                                    #widget_update_bitmask
                                }
                            };
                        ));
                    } else {
                        final_tokens.extend(quote_spanned!(span=>
                            match self.#field_ident.as_mut() {
                                #some_unsafe

                                None => {
                                    self.#field_ident = Some(#widget_init);

                                    {
                                        let _fwc_component_instance = unsafe {
                                            self.#field_ident.as_mut().unwrap_unchecked()
                                        };
                                        #distributor
                                    }

                                    #widget_update_bitmask
                                }
                            };
                        ));
                    }
                }
            }

            // Финализация
            if description.is_maybe.is_some() {
                #[cfg(feature = "safety")]
                if !is_component {
                    self.tokens.push(quote_spanned!(span=>
                        #instance_ident_upper.with(|inst| {
                            let mut _fwc_inst = inst.borrow_mut();
                            match _fwc_inst.#field_ident.as_mut() {
                                Some(_fwc_wb_1) => {
                                    if #condition_statement {
                                        _fwc_wb_1.visible(true);
                                    } else {
                                        _fwc_wb_1.visible(false);
                                    }
                                },
                                None => {},
                            };
                        });
                    ));
                }

                #[cfg(feature = "safety")]
                if is_component {
                    self.tokens.push(quote_spanned!(span=>
                        match self.#field_ident.as_mut() {
                            Some(_fwc_wb_1) => {
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

                #[cfg(not(feature = "safety"))]
                if !is_component {
                    self.tokens.push(quote_spanned!(span=>
                        match #_match_value {
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

                #[cfg(not(feature = "safety"))]
                if is_component {
                    self.tokens.push(quote_spanned!(span=>
                        match self.#field_ident.as_mut() {
                            Some(_fwc_wb_1) => {
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
