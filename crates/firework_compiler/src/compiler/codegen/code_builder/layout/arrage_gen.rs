// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use crate::compiler::codegen::ir::WidgetDescription;

use super::super::*;

impl CodeBuilder {
    pub fn generate_arrange(
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

        // TODO: Добавить реактивонсть
        let mut spacing_tokens = quote! { 0 };
        if let Some(descriptor) = option_descriptor {
            for (field_name, field) in &descriptor.fields {
                if field_name == "spacing" {
                    spacing_tokens = field.token_stream.clone();
                }
            }
        }

        let parent_pos_x = format_ident!("_fwc_pos_x_{}", depth);
        let parent_pos_y = format_ident!("_fwc_pos_y_{}", depth);

        let mut children_arrange = TokenStream::new();

        for (index, child) in children.iter().enumerate() {
            let child_field = format_ident!("_fwc_widget_object_{}", child.id);
            let child_size_ident = format_ident!("_fwc_child_size_{}_{}", depth, index);

            let access_widget = self.generate_arrange_access_widget(
                span,
                &instance_ident_upper,
                &child_field,
                is_component,
            );

            // Вычисляем координаты и сдвигаем курсор лайаута в зависимости от оси лайаута
            // (горизонтальный, вертикальный, стэк и так далее)
            let arrange_logic = match layout_name {
                "hstack" => self.generate_arrange_hstack(
                    span,
                    &child_size_ident,
                    &spacing_tokens,
                    &access_widget,
                ),

                "zstack" => self.generate_arrange_zstack(span, &access_widget),

                // По умолчанию vstack (как и в проходе измерения)
                _ => self.generate_arrange_vstack(
                    span,
                    &child_size_ident,
                    &spacing_tokens,
                    &access_widget,
                ),
            };

            children_arrange.extend(quote_spanned! {span=>
                #arrange_logic
            });
        }

        quote_spanned! {span=>
            // Стартовые курсоры для этого лайаута
            let mut _fwc_current_x = #parent_pos_x;
            let mut _fwc_current_y = #parent_pos_y;
            let mut _fwc_has_visible_children_arrange = false;

            #children_arrange
        }
    }

    fn generate_arrange_access_widget(
        &self,
        span: Span,
        instance_ident_upper: &syn::Ident,
        child_field: &syn::Ident,
        is_component: bool,
    ) -> TokenStream {
        let some_pos_tokens = quote! {
            firework_ui::std_widgets::widget::Widget::position(_fwc_wb_4, (_child_x, _child_y))
        };

        // Если виджета нет, то позиционировать нечего
        let else_pos_tokens = quote! {};

        if is_component {
            quote_spanned! {span=>
                if let Some(ref mut _fwc_wb_4) = self.#child_field {
                    #some_pos_tokens;
                } else {
                    #else_pos_tokens
                }
            }
        } else {
            #[cfg(feature = "safety")]
            {
                quote_spanned! {span=>
                    #instance_ident_upper.with(|inst| {
                        let mut _inst = inst.borrow_mut();
                        if let Some(ref mut _fwc_wb_4) = _inst.#child_field {
                            #some_pos_tokens;
                        } else {
                            #else_pos_tokens
                        }
                    });
                }
            }

            #[cfg(not(feature = "safety"))]
            {
                quote_spanned! {span=>
                    unsafe {
                        let _inst = &mut *::core::ptr::addr_of_mut!(#instance_ident_upper);
                        if let Some(ref mut _fwc_wb_4) = _inst.#child_field {
                            #some_pos_tokens;
                        } else {
                            #else_pos_tokens
                        }
                    }
                }
            }
        }
    }
}
