// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use quote::quote;

use super::super::*;

use crate::compiler::codegen::ir::ComponentDeclaration;

/// Структура для хранения всех токенов методов реализации трейта Widget для компонента
#[derive(Default)]
pub struct ComponentImpl {
    // Токены для метода get_z_size. Он возвращает i32 и нужен чтобы получить сколько
    // z занимает в иерархии виджет
    pub get_z_size_tokens: TokenStream,

    // Токены для метода set_z. Он принимает i32 (z координата) и нужен чтобы установить z
    // координату виджету
    pub set_z_tokens: TokenStream,

    // Токены для метода отдельного от реализации трейта widget чтобы проверить есть ли
    // изменения в битовых масках глобальных пропов компонента
    pub is_dirty_check: TokenStream,

    // Токены для метода видимости реализации трейта Widget, он устанавливает видимость всем
    // виджетам и компонентам внутри компонента
    pub visible_tokens: TokenStream,

    // Токены размонтирования, нужны для компонентов внутри DynList и размонтируют все виджеты
    // и компоненты внутри компонента
    pub unmount_tokens: TokenStream,

    // Токены для метода получения ивента и реакции на него
    pub event_tokens: TokenStream,

    // Токены для обработки события смены кадра
    pub tick_tokens: TokenStream,
}

impl ComponentImpl {
    /// Сгенерировать все токены методов реализации трейта Widget
    pub fn generate(declaration: &ComponentDeclaration, visitor: &CodegenVisitor) -> Self {
        Self {
            get_z_size_tokens: visitor.gen_get_z_size_tokens(declaration),
            set_z_tokens: visitor.gen_set_z_tokens(declaration),
            is_dirty_check: visitor.gen_is_dirty_check(declaration),
            visible_tokens: visitor.gen_visible_tokens(declaration),
            unmount_tokens: visitor.gen_unmount_tokens(declaration),
            event_tokens: visitor.gen_event_tokens(declaration),
            tick_tokens: visitor.gen_tick_tokens(declaration),
        }
    }
}

impl CodegenVisitor<'_> {
    /// Метод для генерации токенов get_z_size у компонента для подсчёта размера всех виджетов
    /// компонента. Требует существования счётчика _fwc_z в области видимости, куда
    /// вставляется результат выполнения метода. Принимает
    pub fn gen_get_z_size_tokens(&self, declaration: &ComponentDeclaration) -> TokenStream {
        let mut z_compute_tokens = quote! {};
        let z_compute_widget_tokens = quote! {
            _fwc_z += firework_ui::std_widgets::widget::Widget::get_z_size(_fwc_element) + 1;
        };

        let z_compute_dynlist_tokens = quote! {
            _fwc_z += _fwc_element.len() + 1;
        };

        for i in &declaration.widgets.widgets {
            let name = format_ident!("{}", i.0);
            let is_microruntime = i.2;

            let z_variant = if is_microruntime {
                &z_compute_dynlist_tokens
            } else {
                &z_compute_widget_tokens
            };

            z_compute_tokens.extend(quote! {
                if let Some(ref mut _fwc_element) = self.#name {
                    #z_variant
                }
            });
        }

        z_compute_tokens
    }

    /// Метод для того, чтобы установить z индекс всем элементам компонента
    pub fn gen_set_z_tokens(&self, declaration: &ComponentDeclaration) -> TokenStream {
        let mut z_compute_tokens = quote! {};
        let z_compute_widget_tokens = quote! {
            firework_ui::std_widgets::widget::Widget::set_z(_fwc_element, _fwc_z);
            _fwc_z += firework_ui::std_widgets::widget::Widget::get_z_size(_fwc_element) + 1;
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
                if let Some(ref mut _fwc_element) = self.#name {
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

    pub fn gen_visible_tokens(&self, declaration: &ComponentDeclaration) -> TokenStream {
        let mut visible_tokens = quote! {};

        for i in &declaration.widgets.widgets {
            let name = format_ident!("{}", i.0);

            visible_tokens.extend(quote! {
                if let Some(ref mut _fwc_element) = self.#name {
                    _fwc_element.visible(visible);
                }
            });
        }

        visible_tokens
    }

    pub fn gen_unmount_tokens(&self, declaration: &ComponentDeclaration) -> TokenStream {
        let mut visible_tokens = quote! {};

        for i in &declaration.widgets.widgets {
            let name = format_ident!("{}", i.0);

            visible_tokens.extend(quote! {
                if let Some(_fwc_element) = self.#name {
                    _fwc_element.unmount();
                }
            });
        }

        visible_tokens
    }

    pub fn gen_event_tokens(&self, declaration: &ComponentDeclaration) -> TokenStream {
        let null_tokens = quote! {};
        let flash_tokens = quote! {
            self.flash(_context);
        };

        let on_click_tokens = if declaration.events.on_click {
            &flash_tokens
        } else {
            &null_tokens
        };

        let on_timeout_tokens = if declaration.events.on_timeout {
            &flash_tokens
        } else {
            &null_tokens
        };

        let mut visible_tokens = quote! {
            if firework_ui::tiny_matches!(_context.cycle, firework_ui::LifeCycle::Event) {
                match firework_ui::take_current_event() {
                    firework_ui::CurrentEvent::Touch {
                        hit_object_id: Some(id),
                        phase,
                        ..
                    } => {
                        #on_click_tokens
                    }
                    firework_ui::CurrentEvent::Timeout { .. } => {
                        #on_timeout_tokens
                    }
                    _ => {}
                }
            }
        };

        for i in &declaration.widgets.widgets {
            let name = format_ident!("{}", i.0);

            visible_tokens.extend(quote! {
                if let Some(ref mut _fwc_element) = self.#name {
                    _fwc_element.__event(_context);
                }
            });
        }

        visible_tokens
    }

    pub fn gen_tick_tokens(&self, declaration: &ComponentDeclaration) -> TokenStream {
        let mut tick_tokens = quote! {};

        for i in &declaration.widgets.widgets {
            let name = format_ident!("{}", i.0);
            let resolver = self.gen_tick_resolver(&i.4);

            tick_tokens.extend(quote! {
                if let Some(ref mut _fwc_element) = self.#name {
                    let _fwc_wb_2 = _fwc_element.__tick(_delta);
                    #resolver
                }
            });
        }

        tick_tokens
    }
}
