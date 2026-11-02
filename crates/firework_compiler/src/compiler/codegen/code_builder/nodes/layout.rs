// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use crate::CompileType;
use crate::compiler::codegen::ir::WidgetDescription;

use super::super::*;

impl CodeBuilder {
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(span = ?span)))]
    pub fn node_layout(
        &mut self,
        span: Span,
        final_tokens: &mut TokenStream,
        statement: &FireworkStatement,
        visitor: &mut CodegenVisitor,
        processed_body: &TokenStream,
    ) -> bool {
        if let FireworkAction::LayoutBlock(name, _microruntime, descriptor, depth, children) =
            &statement.action
        {
            // Хелпер это инструмент разработчика, для него не нужно генерировать лайаут
            if name == "helper" {
                return false;
            }

            let depth = *depth;
            let is_component = matches!(visitor.flags.compile_type, CompileType::Component);

            // Локальные счётчики для этого лайаута
            let current_constraints = format_ident!("_fwc_constraints_{}", depth);
            let current_pos_x = format_ident!("_fwc_pos_x_{}", depth);
            let current_pos_y = format_ident!("_fwc_pos_y_{}", depth);

            let init_layout = if depth == 1 {
                let root_constraints_init = if is_component {
                    // Для компонента берём размер из его подложки, этот размер устаналивается
                    // пропом size (методом __set_size внутри). Нужно использовать именно эти
                    // размеры, а не глобальные размеры окна, так как компонент это только
                    // часть экрана или другого компонента
                    quote_spanned! {span=>
                        let (_fwc_root_w, _fwc_root_h) = if let Some(_c) = &self._fwc__fwc_component {
                            _c.size
                        } else {
                            (0, 0)
                        };
                    }
                } else {
                    // Для экрана запрашиваем размер у адаптера синхронно
                    quote_spanned! {span=>
                        let (_fwc_root_w, _fwc_root_h) = if let firework_ui::AdapterResult::Size(w, h) =
                            firework_ui::adapter_command(firework_ui::AdapterCommand::GetViewport)
                        {
                            (w as i32, h as i32)
                        } else {
                            (0, 0)
                        };
                    }
                };

                quote_spanned! {span=>
                    #root_constraints_init

                    let mut #current_constraints = firework_ui::layout::Constraints {
                        min_width: 0,
                        max_width: _fwc_root_w,
                        min_height: 0,
                        max_height: _fwc_root_h,
                    };
                    let mut #current_pos_x = 0;
                    let mut #current_pos_y = 0;
                }
            } else {
                // Для глубина лайаута в иерархии больше 1, то есть это вложенный лауаут, то
                // берем ограничения от предыдущего уровня глубины
                let previous_depth = depth - 1;
                let previous_constraints = format_ident!("_fwc_constraints_{}", previous_depth);
                let previous_pos_x = format_ident!("_fwc_pos_x_{}", previous_depth);
                let previous_pos_y = format_ident!("_fwc_pos_y_{}", previous_depth);

                quote_spanned! {span=>
                    let mut #current_constraints = #previous_constraints;
                    let mut #current_pos_x = #previous_pos_x;
                    let mut #current_pos_y = #previous_pos_y;
                }
            };

            let measure_tokens = self.generate_measure(
                span,
                statement.screen_index,
                depth,
                is_component,
                descriptor,
                children,
            );

            final_tokens.extend(quote_spanned!(span=>
                {
                    // println!("Layout begin placeholder");
                    #init_layout

                    #processed_body

                    // Расчёт размеров всех виджетов
                    #measure_tokens

                    // println!("Layout end placeholder");
                }
            ));

            return true;
        }

        false
    }

    fn generate_measure(
        &self,
        span: Span,
        screen_index: u128,
        depth: usize,
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
            // TODO: Начать учитывать реактивные пропы

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
            (
                /* Размер */ firework_ui::std_widgets::widget::Widget::layout(_fwc_wb_4, _c),
                /* Видимость */ firework_ui::std_widgets::widget::Widget::is_layout_visible(_fwc_wb_4)
            )
        };

        // Размер если виджет не найден при попытке получить его из Option в экране/компоненте
        // (То есть, если он не создан, то не учавствует)
        let else_size_tokens = quote! {
            (
                firework_ui::layout::Size { width: 0, height: 0 },
                false,
            )
        };

        for (index, child) in children.iter().enumerate() {
            let child_field = format_ident!("_fwc_widget_object_{}", child.id);
            let child_size_ident = format_ident!("_fwc_child_size_{}_{}", depth, index);
            let child_visible_ident = format_ident!("_fwc_child_vis_{}_{}", depth, index);

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
            children_measure.extend(quote_spanned! {span=>
                let (#child_size_ident, #child_visible_ident) = {
                    let _c = firework_ui::layout::Constraints {
                        min_width: 0,
                        max_width: #parent_constraints.max_width,
                        min_height: 0,
                        max_height: #parent_constraints.max_height.saturating_sub(_fwc_current_height).max(0),
                    };

                    #access_widget
                };

                if #child_visible_ident {
                    // Размер увеличивается только если виджет видимый
                    if _fwc_has_visible_children {
                        _fwc_current_height += #spacing_tokens;
                    }

                    _fwc_current_height += #child_size_ident.height;
                    _fwc_max_width = _fwc_max_width.max(#child_size_ident.width);
                    _fwc_has_visible_children = true;
                }
            });
        }

        // Возвращает итоговый расчёт размеров
        quote_spanned! {span=>
            let mut _fwc_current_height: i32 = 0;
            let mut _fwc_max_width: i32 = 0;

            // Флаг, который стаивтся на true если появляются видимые виджеты. Это необходимо
            // чтобы не давать первому виджету спейсинг, но первые виджеты могут быть скрыты
            let mut _fwc_has_visible_children = false;

            #children_measure

            let #layout_size_ident = firework_ui::layout::Size {
                width: _fwc_max_width,
                height: _fwc_current_height,
            };
        }
    }
}
