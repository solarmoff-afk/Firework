// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub use super::super::*;

impl Analyzer {
    /// Метод для вывода всего что собранно в области видимости
    pub fn log_scope(&self) {
        #[cfg(feature = "debug_output")]
        println!("{:#?}", self.lifetime_manager.scope.variables);
    }

    pub fn update_scope(&mut self, scope: Scope, set_scope: bool) {
        let base_stmt = self.context.statement.clone();
        let drop_statements = self
            .lifetime_manager
            .update_scope(scope, set_scope, &base_stmt);

        for stmt in drop_statements {
            self.context.ir.push(stmt);
            self.statement_index += 1;
        }
    }
}
