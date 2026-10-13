// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use syn::{ImplItem, ItemFn, ItemImpl, ReturnType, Type, parse_quote};

use super::DesugarVisitor;
use crate::CompileType;

impl DesugarVisitor {
    pub(crate) fn desugar_flash_item_fn(&mut self, i: &mut ItemFn) {
        i.block.stmts.push(parse_quote! {
            firework_ui::_fwrt_terminator!();
        });
    }

    pub(crate) fn desugar_flash_item_impl(&mut self, i: &mut ItemImpl) {
        if !matches!(self.flags.compile_type, CompileType::Component) {
            return;
        }

        for item in &mut i.items {
            if let ImplItem::Fn(method) = item {
                if method.sig.ident == "flash" && Self::is_valid_flash_return(&method.sig.output) {
                    method.block.stmts.push(parse_quote! {
                        firework_ui::_fwrt_terminator!();
                    });
                }
            }
        }
    }

    fn is_valid_flash_return(output: &ReturnType) -> bool {
        match output {
            ReturnType::Default => true,
            ReturnType::Type(_, ty) => match ty.as_ref() {
                Type::Tuple(tuple) => tuple.elems.is_empty(),
                Type::Never(_) => true,
                _ => false,
            },
        }
    }
}
