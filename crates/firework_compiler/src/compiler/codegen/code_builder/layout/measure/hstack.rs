// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use crate::compiler::codegen::code_builder::CodeBuilder;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote_spanned;

impl CodeBuilder {
    /// Горизонтальный стэк. Виджеты и компоненты размещаются по горизонтальной линнии
    ///
    /// [ - - - - - - - - - - - - - ]
    /// [ Elem      Elem       Elem ]
    /// ] - - - - - - - - - - - - - ]
    pub fn generate_measure_hstack(
        &self,
        span: Span,
        parent_constraints: &syn::Ident,
        child_size_ident: &syn::Ident,
        spacing_tokens: &TokenStream,
    ) -> (TokenStream, TokenStream) {
        let constraints_tokens = quote_spanned! {span=>
            firework_ui::layout::Constraints {
                min_width: 0,
                max_width: #parent_constraints.max_width.saturating_sub(_fwc_layout_width).max(0),
                min_height: 0,
                max_height: #parent_constraints.max_height,
            }
        };

        let accumulate_tokens = quote_spanned! {span=>
            if #child_size_ident.width > 0 {
                if _fwc_has_visible_children {
                    // Тут мы добавляем ширину к размеру
                    _fwc_layout_width += #spacing_tokens;
                }

                _fwc_layout_width += #child_size_ident.width;
                _fwc_layout_height = _fwc_layout_height.max(#child_size_ident.height);

                // Этот флаг помечает, что в лайауте уже был один видимый виджет
                // или компонент и теперь можно считать отступы, ибо от невидимых
                // виджетов и компонентов отступы считать не нужно
                _fwc_has_visible_children = true;
            }
        };

        (constraints_tokens, accumulate_tokens)
    }
}
