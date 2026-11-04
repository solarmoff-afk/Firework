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
    pub fn generate_measure_vstack(
        &self,
        span: Span,
        parent_constraints: &syn::Ident,
        child_size_ident: &syn::Ident,
        spacing_tokens: &TokenStream,
    ) -> (TokenStream, TokenStream) {
        let constraints_tokens = quote_spanned! {span=>
            firework_ui::layout::Constraints {
                min_width: 0,
                max_width: #parent_constraints.max_width,
                min_height: 0,
                max_height: #parent_constraints.max_height.saturating_sub(_fwc_layout_height).max(0),
            }
        };

        let accumulate_tokens = quote_spanned! {span=>
            if #child_size_ident.height > 0 {
                if _fwc_has_visible_children {
                    // Тут мы добавляем высоту к размеру
                    _fwc_layout_height += #spacing_tokens;
                }

                _fwc_layout_height += #child_size_ident.height;
                _fwc_layout_width = _fwc_layout_width.max(#child_size_ident.width);
                _fwc_has_visible_children = true;
            }
        };

        (constraints_tokens, accumulate_tokens)
    }
}
