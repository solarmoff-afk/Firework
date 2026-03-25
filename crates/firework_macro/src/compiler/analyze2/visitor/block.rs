// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub use super::super::*;

impl<'ast> Analyzer {
    /// Вход в новую область видимости
    pub(crate) fn analyze_block(&mut self, i: &'ast syn::Block) {
        // Сначала клонируем всё состояние текущей области видимости, когда эта область
        // видимости закончится (та, что сейчас открывается) все переменные и не только
        // созданные внутри неё будут дропнуты и мы не можем их использовать. После
        // завершения блока нам нужно вернуть ранее сохранённое состояние, а для этого
        // мы будем использовать клон который создаётся здесь
        let scope = self.scope.clone();

        // Парсинг области видимости, переменные созданные в этой области видимости будут
        // в self.scrope.variables
        visit::visit_block(self, i);

        // Дебаг вывод всех переменных который собрали в этой новой области видимости
        self.log_scope();

        // Область видимости закончилась, нужно восстановить состояние используя клон
        self.scope = scope;
    }

    /// Условие
    pub(crate) fn analyze_expr_if(&mut self, i: &'ast ExprIf) {
        let sparks = self.get_sparks(&i.cond);
        let condition_code = i.cond.to_token_stream().to_string();
        
        self.handle_reactive_block(
            sparks.clone(),
            false,
            format!("if {} {{", condition_code),
            FireworkAction::ReactiveIf(sparks),
            |this| visit::visit_expr_if(this, i),
        );
    }

    /// Цикл while
    pub(crate) fn analyze_expr_while(&mut self, i: &'ast ExprWhile) {
        let sparks = self.get_sparks(&i.cond);
        let condition_code = i.cond.to_token_stream().to_string();
        
        self.handle_reactive_block(
            sparks.clone(),
            true,
            format!("while {} {{", condition_code),
            FireworkAction::ReactiveWhile(sparks.clone()),
            |this| visit::visit_expr_while(this, i),
        );
    }
    
    /// Цикл for
    pub(crate) fn analyze_expr_for_loop(&mut self, i: &'ast ExprForLoop) {
        let sparks = self.get_sparks(&i.expr);
        let pattern_code = i.pat.to_token_stream().to_string();
        let expr_code = i.expr.to_token_stream().to_string();
        
        self.handle_reactive_block(
            sparks.clone(),
            true,
            format!("for {} in {} {{", pattern_code, expr_code),
            FireworkAction::ReactiveFor(sparks.clone()),
            |this| visit::visit_expr_for_loop(this, i),
        );
    }
    
    /// Match
    pub(crate) fn analyze_expr_match(&mut self, i: &'ast ExprMatch) {
        let sparks = self.get_sparks(&i.expr);
        let expr_code = i.expr.to_token_stream().to_string();
        
        self.handle_reactive_block(
            sparks.clone(),
            false,
            format!("match {} {{", expr_code),
            FireworkAction::ReactiveMatch(sparks.clone()),
            |this| visit::visit_expr_match(this, i),
        );
    }

    /// Loop { ... }
    pub(crate) fn analyze_expr_loop(&mut self, i: &'ast ExprLoop) {
        self.handle_reactive_block(
            Vec::new(),
            true,
            "loop {".to_string(),
            FireworkAction::DefaultCode,
            |this| visit::visit_expr_loop(this, i),
        );
    }
}
