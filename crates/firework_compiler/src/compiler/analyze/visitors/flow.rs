// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub use super::super::*;

impl<'ast> Analyzer {
    /// Выход из цикла (break), пример:
    /// for i in 1..5 {
    ///  break;
    /// }
    ///
    /// Также поддерживает break 'label
    pub(crate) fn analyze_expr_break(&mut self, i: &'ast ExprBreak) {
        let target_scope = self.get_target_scope(&i.label);
        self.update_scope(target_scope, false);

        visit::visit_expr_break(self, i);
    }

    /// Шаг цикла (continue)
    pub(crate) fn analyze_expr_continue(&mut self, i: &'ast ExprContinue) {
        let target_scope = self.get_target_scope(&i.label);
        self.update_scope(target_scope, false);

        visit::visit_expr_continue(self, i);
    }

    /// Возврат значения (return)
    pub(crate) fn analyze_expr_return(&mut self, i: &'ast ExprReturn) {
        // При входе в функцию было сохранение области видимости в item_scope,
        // когда идёт выход из функции нужно сгенерировать возврат владения
        // для всех спарков которые были арендованы (take) от BSS экземпляра,
        // для этого подойдёт update_scope, в качестве второй точки для диффинга
        // используется item_scope
        let target_scope = self.lifetime_manager.item_scope.clone();

        self.update_scope(target_scope, false);

        visit::visit_expr_return(self, i);
    }
}
