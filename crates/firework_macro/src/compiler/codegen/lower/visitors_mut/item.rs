// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub use super::*;

impl LowerVisitor<'_> {
    pub(crate) fn lower_file_mut(&mut self, i: &mut File) {
        let mut new_items = Vec::new();

        // Забираем элементы, чтобы не клонировать весь вектор сразу
        let items = std::mem::take(&mut i.items);

        for item in items {
            match item {
                Item::Fn(mut item_fn) => {
                    self.lower_ui_function(&mut item_fn.sig, &mut item_fn.block);

                    new_items.push(Item::Fn(item_fn));
                }

                Item::Impl(mut item_impl) => {
                    for item in &mut item_impl.items {
                        if let ImplItem::Fn(method) = item
                            && method.sig.ident == "flash"
                        {
                            self.lower_ui_function(&mut method.sig, &mut method.block);
                        }
                    }
                    new_items.push(Item::Impl(item_impl));
                }

                mut other_item => {
                    self.visit_item_mut(&mut other_item);
                    new_items.push(other_item);
                }
            }
        }

        i.items = new_items;
    }

    fn lower_ui_function(&mut self, sig: &mut Signature, block: &mut Block) {
        let function_name = sig.ident.to_string();
        self.ui_id = self
            .ir
            .screens
            .iter()
            .find(|(name, _, _)| name == &function_name)
            .map(|(_, _, id)| *id);

        self.visit_block_mut(block);
    }
}
