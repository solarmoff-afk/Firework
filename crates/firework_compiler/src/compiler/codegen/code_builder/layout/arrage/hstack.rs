// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use crate::compiler::codegen::code_builder::CodeBuilder;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote_spanned;

impl CodeBuilder {
    /// Горизонтальный стэк. Виджеты и компоненты размещаются по горизонтальной линии
    ///
    /// [ - - - - - - - - - - - - - ]
    /// [ Elem      Elem       Elem ]
    /// ] - - - - - - - - - - - - - ]
    pub fn generate_arrange_hstack(
        &self,
        span: Span,
        child_size_ident: &syn::Ident,
        spacing_tokens: &TokenStream,
        access_widget: &TokenStream,
    ) -> TokenStream {
        quote_spanned! {span=>
            if #child_size_ident.width > 0 {
                if _fwc_has_visible_children_arrange {
                    _fwc_current_x += #spacing_tokens;
                }

                // TODO: Добавить CrossAxisAlignment
                let _child_x = _fwc_current_x;

                let _child_y = _fwc_current_y;

                #access_widget

                _fwc_current_x += #child_size_ident.width;
                _fwc_has_visible_children_arrange = true;
            }
        }
    }
}
