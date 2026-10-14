// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use quote::quote;

use super::super::*;

use crate::compiler::codegen::ir::SpecialWidget;
use crate::compiler::codegen::ir::WidgetDescription;

impl CodegenVisitor<'_> {
    pub fn gen_tick_resolver(&self, description: &WidgetDescription) -> TokenStream {
        let mut resolver = quote! { None };

        if let SpecialWidget::Timer(_timer) = &description.special {
            resolver = quote! {
                if _fwc_wb_2 {
                    let _fwc_wb_3 = _fwc_element.id.get();
                    println!("Id: {}, Depth: {}", _fwc_wb_3.0, _fwc_wb_3.1);
                }
            };
        }

        resolver
    }
}
