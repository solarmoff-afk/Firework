// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use crate::compiler::codegen::ir::WidgetDescription;

use super::super::*;

impl CodeBuilder {
    pub fn generate_measure(
        &self,
        span: Span,
        screen_index: u128,
        depth: usize,
        layout_name: &str,
        is_component: bool,
        option_descriptor: &Option<WidgetDescription>,
        children: &[WidgetDescription],
    ) -> TokenStream {
        let instance_ident_upper =
            format_ident!("APPLICATIONUIBLOCKSTRUCT{}_INSTANCE", screen_index);

        // Извлечённые поля лайаута. Они создаются с значением которое будет по умолчанию
        // если нет лайаут дескриптора layout! {}
        let mut spacing_tokens = quote! { 0 };

        // Извлечение полей лайаут
        if let Some(descriptor) = option_descriptor {
            for (field_name, field) in &descriptor.fields {
                if field_name == "spacing" {
                    spacing_tokens = field.token_stream.clone();
                }

                // TODO: Добавить остальные поля
            }
        }

        let parent_constraints = format_ident!("_fwc_constraints_{}", depth);
        let layout_size_ident = format_ident!("_fwc_layout_size_{}", depth);

        let mut children_measure = TokenStream::new();

        // Токены если виджет существует (просто расчёт лайаута). Виджет (в if let Some) должен
        // называться _fwc_wb_4
        let some_size_tokens = quote! {
            firework_ui::std_widgets::widget::Widget::layout(_fwc_wb_4, _c)
        };

        // Размер если виджет не найден при попытке получить его из Option в экране/компоненте
        // (То есть, если он не создан, то не учавствует)
        let else_size_tokens = quote! {
            firework_ui::layout::Size { width: 0, height: 0 }
        };

        for (index, child) in children.iter().enumerate() {
            let child_field = format_ident!("_fwc_widget_object_{}", child.id);
            let child_size_ident = format_ident!("_fwc_child_size_{}_{}", depth, index);

            let access_widget = if is_component {
                quote_spanned! {span=>
                    if let Some(ref mut _fwc_wb_4) = self.#child_field {
                        #some_size_tokens
                    } else {
                        #else_size_tokens
                    }
                }
            } else {
                #[cfg(feature = "safety")]
                {
                    quote_spanned! {span=>
                        #instance_ident_upper.with(|inst| {
                            let mut _inst = inst.borrow_mut();
                            if let Some(ref mut _fwc_wb_4) = _inst.#child_field {
                                #some_size_tokens
                            } else {
                                #else_size_tokens
                            }
                        })
                    }
                }

                #[cfg(not(feature = "safety"))]
                {
                    quote_spanned! {span=>
                        unsafe {
                            let _inst = &mut *::core::ptr::addr_of_mut!(#instance_ident_upper);
                            if let Some(ref mut _fwc_wb_4) = _inst.#child_field {
                                #some_size_tokens
                            } else {
                                #else_size_tokens
                            }
                        }
                    }
                }
            };

            // Генерация добавления размера к счётчикам лайаута
            let (constraints_tokens, accumulate_tokens) = match layout_name {
                // Горизонтальный стэк. Виджеты и компоненты размещаются по горизонтальной
                // линнии
                //
                // [ - - - - - - - - - - - - - ]
                // [ Elem      Elem       Elem ]
                // ] - - - - - - - - - - - - - ]
                "hstack" => self.generate_measure_hstack(
                    span,
                    &parent_constraints,
                    &child_size_ident,
                    &spacing_tokens,
                ),

                // Обычный стэк в котором элементы накладываются друг на друга по слоям
                "zstack" => {
                    self.generate_measure_zstack(span, &parent_constraints, &child_size_ident)
                }

                _ => self.generate_measure_vstack(
                    span,
                    &parent_constraints,
                    &child_size_ident,
                    &spacing_tokens,
                ),
            };

            children_measure.extend(quote_spanned! {span=>
                let #child_size_ident = {
                    let _c = #constraints_tokens;
                    #access_widget
                };

                #accumulate_tokens
            });
        }

        // Возвращает итоговый расчёт размеров
        quote_spanned! {span=>
            let mut _fwc_layout_width: i32 = 0;
            let mut _fwc_layout_height: i32 = 0;
            let mut _fwc_has_visible_children = false;

            #children_measure

            let #layout_size_ident = firework_ui::layout::Size {
                width: _fwc_layout_width,
                height: _fwc_layout_height,
            };
        }
    }
}
