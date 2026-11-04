// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use crate::compiler::codegen::code_builder::CodeBuilder;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote_spanned;

impl CodeBuilder {
    /// Обычный стэк в котором элементы накладываются друг на друга по слоям
    pub fn generate_measure_zstack(
        &self,
        span: Span,
        parent_constraints: &syn::Ident,
        child_size_ident: &syn::Ident,
    ) -> (TokenStream, TokenStream) {
        let constraints_tokens = quote_spanned! {span=>
            firework_ui::layout::Constraints {
                min_width: 0,
                max_width: #parent_constraints.max_width,
                min_height: 0,
                max_height: #parent_constraints.max_height,
            }
        };

        let accumulate_tokens = quote_spanned! {span=>
            _fwc_layout_width = _fwc_layout_width.max(#child_size_ident.width);
            _fwc_layout_height = _fwc_layout_height.max(#child_size_ident.height);
        };

        (constraints_tokens, accumulate_tokens)
    }
}
