// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub use super::super::*;

impl<'ast> Analyzer {
    /// Замыкание
    pub(crate) fn analyze_expr_closure(&mut self, i: &'ast ExprClosure) {
        let old_in_closure = self.lifetime_manager.in_closure;

        if !self.lifetime_manager.in_closure {
            self.lifetime_manager.in_closure = true;
        }

        visit::visit_expr(self, &i.body);
        self.lifetime_manager.in_closure = old_in_closure;
    }
}
