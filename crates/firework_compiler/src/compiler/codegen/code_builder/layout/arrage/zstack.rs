// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use crate::compiler::codegen::code_builder::CodeBuilder;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote_spanned;

impl CodeBuilder {
    /// Обычный стэк в котором элементы накладываются друг на друга по слоям
    pub fn generate_arrange_zstack(&self, span: Span, access_widget: &TokenStream) -> TokenStream {
        quote_spanned! {span=>
            // TODO: Добавить Alignment
            let _child_x = _fwc_current_x;

            let _child_y = _fwc_current_y;

            #access_widget
        }
    }
}
