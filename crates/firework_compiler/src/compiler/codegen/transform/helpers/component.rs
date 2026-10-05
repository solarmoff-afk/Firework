// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use quote::quote;

use super::super::*;

use crate::compiler::codegen::ir::ComponentDeclaration;

impl CodegenVisitor<'_> {
    /// Метод для генерации токенов get_z_size у компонента для подсчёта размера всех виджетов
    /// компонента. Требует существования счётчика _fwc_z в области видимости, куда
    /// вставляется результат выполнения метода. Принимает
    pub fn gen_get_z_size_tokens(&self, declaration: &ComponentDeclaration) -> TokenStream {
        let mut z_compute_tokens = quote! {};
        let z_compute_widget_tokens = quote! {
            _fwc_z += firework_ui::std_widgets::widget::Widget::get_z_size(&_fwc_element) + 1;
        };

        let z_compute_dynlist_tokens = quote! {
            _fwc_z += _fwc_element.len() + 1;
        };

        for i in &declaration.widgets.widgets {
            let name = format_ident!("{}", i.0);
            let is_microruntime = i.2;

            if
            /* has_z */
            i.3 {
                continue;
            }

            let z_variant = if is_microruntime {
                &z_compute_dynlist_tokens
            } else {
                &z_compute_widget_tokens
            };

            z_compute_tokens.extend(quote! {
                if let Some(_fwc_element) = self.#name {
                    #z_variant
                }
            });
        }

        z_compute_tokens
    }

    /// Метод для того, чтобы установить z индекс всем элементам компонента
    pub fn gen_set_s_tokens(&self, declaration: &ComponentDeclaration) -> TokenStream {
        let mut z_compute_tokens = quote! {};
        let z_compute_widget_tokens = quote! {
            firework_ui::std_widgets::widget::Widget::set_z(&_fwc_element, _fwc_z);
            _fwc_z += firework_ui::std_widgets::widget::Widget::get_z_size(&_fwc_element) + 1;
        };

        let z_compute_dynlist_tokens = quote! {
            _fwc_element.set_z_range(_fwc_z);
            _fwc_z += _fwc_element.len() + 1;
        };

        for i in &declaration.widgets.widgets {
            let name = format_ident!("{}", i.0);
            let is_microruntime = i.2;

            if
            /* has_z */
            i.3 {
                continue;
            }

            let z_variant = if is_microruntime {
                &z_compute_dynlist_tokens
            } else {
                &z_compute_widget_tokens
            };

            z_compute_tokens.extend(quote! {
                if let Some(_fwc_element) = self.#name {
                    #z_variant
                }
            });
        }

        z_compute_tokens
    }

    pub fn gen_is_dirty_check(&self, declaration: &ComponentDeclaration) -> TokenStream {
        if declaration.props_counter == 0 {
            return quote! { false };
        }

        let num_masks = (declaration.props_counter + 63) / 64;

        let mut conditions = Vec::new();
        for i in 0..num_masks {
            let mask_name = format_ident!("_fwc_component_bitmask_{}", i);
            conditions.push(quote! { self.#mask_name != 0 });
        }

        let mut result = quote! {};
        for (i, cond) in conditions.into_iter().enumerate() {
            if i == 0 {
                result = cond;
            } else {
                result = quote! { #result || #cond };
            }
        }

        result
    }
}
