// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

#[cfg(feature = "trace")]
use tracing::instrument;

#[cfg(feature = "trace")]
use quote::quote;

pub use super::super::*;

use crate::CompileType;

impl CodegenVisitor<'_> {
    #[instrument(skip_all, fields(node = %quote!(#i)))]
    pub(crate) fn analyze_item_mut(&mut self, i: &mut Item) {
        let mut should_remove = false;

        if let Item::Macro(_macro) = i {
            // В Shared режиме компиляции макрос state явлется просто декларацией глобального
            // состояния на которое функции shared блок могут брать мутабельные ссылки. Это
            // позволяет писать сервисы и провайдеры состояния
            if matches!(self.flags.compile_type, CompileType::Shared)
                && _macro.mac.path.is_ident("state")
            {
                should_remove = true;
            }
        }

        if should_remove {
            *i = Item::Verbatim(Default::default());
        } else {
            visit_mut::visit_item_mut(self, i);
        }
    }
}
