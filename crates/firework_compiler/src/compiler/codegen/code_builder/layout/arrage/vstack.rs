// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use crate::compiler::codegen::code_builder::CodeBuilder;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote_spanned;

impl CodeBuilder {
    /// По умолчанию vstack, то есть вертикальный стэк. В нём используется
    /// компоновка в колонку
    ///
    /// [ - - - - ]
    /// [ Element ]
    /// [         ]
    /// [ Element ]
    /// [         ]
    /// [ Element ]
    /// [ - - - - ]
    pub fn generate_arrange_vstack(
        &self,
        span: Span,
        child_size_ident: &syn::Ident,
        spacing_tokens: &TokenStream,
        cross_alignment: &TokenStream,
        parent_constraints: &syn::Ident,
        access_widget: &TokenStream,
    ) -> TokenStream {
        quote_spanned! {span=>
            if #child_size_ident.height > 0 {
                if _fwc_has_visible_children_arrange {
                    _fwc_current_y += #spacing_tokens;
                }

                let _cross_offset = #cross_alignment.calculate(#parent_constraints.max_width, #child_size_ident.width);
                let _child_x = _fwc_current_x + _cross_offset;
                let _child_y = _fwc_current_y;

                #access_widget

                _fwc_current_y += #child_size_ident.height;
                _fwc_has_visible_children_arrange = true;
            }
        }
    }
}
