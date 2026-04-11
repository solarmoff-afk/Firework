// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use syn::spanned::Spanned;

pub use super::super::*;

impl<'ast> Analyzer {
    /// Вход в новую область видимости
    pub(crate) fn analyze_block(&mut self, i: &'ast syn::Block) { 
        // Сначала клонируем всё состояние текущей области видимости, когда эта область
        // видимости закончится (та, что сейчас открывается) все переменные и не только
        // созданные внутри неё будут дропнуты и мы не можем их использовать. После
        // завершения блока нам нужно вернуть ранее сохранённое состояние, а для этого
        // мы будем использовать клон который создаётся здесь
        self.old_scope.push(self.scope.clone());

        self.scope.is_cycle = false;

        // Парсинг области видимости, переменные созданные в этой области видимости будут
        // в self.scrope.variables
        visit::visit_block(self, i); 
 
        // Дебаг вывод всех переменных который собрали в этой новой области видимости
        self.log_scope();

        // Область видимости закончилась, нужно восстановить состояние используя клон
        let scope = self.old_scope.pop().unwrap_or(Scope::new()); 

        self.update_scope(scope, true);
    }

    /// Условие. Обрабатывает if, else if и else с поддержкой реактивных переменных
    /// в условии
    pub(crate) fn analyze_expr_if(&mut self, i: &'ast ExprIf) {
        self.context.statement.span = i.if_token.span();

        // Спарки в условии
        let sparks = self.get_sparks(&i.cond);

        let mut else_action = FireworkAction::DefaultCode;

        // Если в условии есть спарки то else блок будет реактивным, так как основное
        // условие реактивное
        if sparks.len() > 0 {
            else_action = FireworkAction::ReactiveElse;
        }
        
        // Исходная строка созданная из токенов внутри условия блока
        let condition_code = i.cond.to_token_stream().to_string();
        
        self.handle_reactive_block(
            sparks.clone(),
            false,
            format!("if {} {{", condition_code),
            FireworkAction::ReactiveBlock(FireworkReactiveBlock::ReactiveIf, sparks),
            |this| {
                // Основное тело условия
                visit::visit_block(this, &i.then_branch);
            
                // При наличии else блока
                if let Some((_, else_branch)) = &i.else_branch {
                    match &**else_branch {
                        Expr::If(else_if) => {
                            // Для else if нужно проанализировать его как отдельное условие
                            // со своими собственными спарками
                            let else_if_sparks = this.get_sparks(&else_if.cond);
                            let else_if_condition_code = else_if.cond.to_token_stream().to_string();
                        
                            this.handle_reactive_block(
                                else_if_sparks.clone(),
                                false,
                                format!("}} else if {} {{", else_if_condition_code),
                                FireworkAction::ReactiveBlock(
                                    FireworkReactiveBlock::ReactiveIf, else_if_sparks,
                                ),
                                |inner_this| {
                                    visit::visit_block(inner_this, &else_if.then_branch);
                                    
                                    // Вложенные else/else if
                                    if let Some((_, inner_else_branch)) = &else_if.else_branch {
                                        match &**inner_else_branch {
                                            Expr::If(inner_else_if) => {
                                                inner_this.analyze_expr_if(inner_else_if);
                                            },

                                            syn::Expr::Block(inner_block) => {
                                                inner_this.analyze_block(&inner_block.block);
                                            },
                                        
                                            _ => {}
                                        }
                                    }
                                },
                            );
                        },

                        syn::Expr::Block(else_block) => { 
                            this.handle_reactive_block(
                                Vec::new(),
                                false,
                                "} else {".to_string(),
                                else_action,
                                |inner_this| inner_this.analyze_block(&else_block.block),
                            );
                        },

                        _ => {}
                    }
                }
            },
        );
    }

    /// Цикл while
    pub(crate) fn analyze_expr_while(&mut self, i: &'ast ExprWhile) {
        let sparks = self.get_sparks(&i.cond);
        let condition_code = i.cond.to_token_stream().to_string();

        // Метка цикла
        let label = i.label.as_ref().map(|l| l.name.ident.to_string());

        self.scope.is_cycle = true;
        self.scope.label = label;
       
        self.handle_reactive_block(
            sparks.clone(),
            true,
            format!("while {} {{", condition_code),
            FireworkAction::ReactiveBlock(FireworkReactiveBlock::ReactiveWhile, sparks.clone()),
            |this| visit::visit_expr_while(this, i),
        );
    }
    
    /// Цикл for
    pub(crate) fn analyze_expr_for_loop(&mut self, i: &'ast ExprForLoop) {
        let sparks = self.get_sparks(&i.expr);
        let pattern_code = i.pat.to_token_stream().to_string();
        let expr_code = i.expr.to_token_stream().to_string();

        // Метка цикла
        let label = i.label.as_ref().map(|l| l.name.ident.to_string());

        self.scope.is_cycle = true;
        self.scope.label = label;
        
        self.handle_reactive_block(
            sparks.clone(),
            true,
            format!("for {} in {} {{", pattern_code, expr_code),
            FireworkAction::ReactiveBlock(FireworkReactiveBlock::ReactiveFor, sparks.clone()),
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
            FireworkAction::ReactiveBlock(FireworkReactiveBlock::ReactiveMatch, sparks.clone()),
            |this| visit::visit_expr_match(this, i),
        );
    }

    /// Loop { ... }
    pub(crate) fn analyze_expr_loop(&mut self, i: &'ast ExprLoop) {
        // Метка цикла
        let label = i.label.as_ref().map(|l| l.name.ident.to_string());

        self.scope.is_cycle = true;
        self.scope.label = label;

        self.handle_reactive_block(
            Vec::new(),
            true,
            "loop {".to_string(),
            FireworkAction::DefaultCode,
            |this| visit::visit_expr_loop(this, i),
        );
    }

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
        let target_scope = self.item_scope.clone();
        
        self.update_scope(target_scope, false);
        
        visit::visit_expr_return(self, i);
    }

    /// Этот метод используется в break и continue чтобы найти последнюю область
    /// видимости которая явлется циклом, label нужен для циклов с именем, принимает
    /// опциональный Lifetime от syn, а возвращает область видимости которая была
    /// найдена в стэке
    fn get_target_scope(&mut self, label: &Option<Lifetime>) -> Scope {
        // Получение последней области видимости в стэке
        let target_scope = if let Some(label_break) = label {
            // Имя цикла который нужно остановить
            let label_name = label_break.ident.to_string();

            // Поиск цикла с таким именем по стэку областей видимости
            self.old_scope.iter()
                .rev()
                .find(|s| s.label.as_ref() == Some(&label_name))
                .cloned()
                .unwrap_or_else(|| Scope::new())
        } else {
            // Если нет имени цикла в break {'имя} <- вот тут
            self.old_scope.iter()
                .rev()
                .find(|s| s.is_cycle)
                .cloned()
                .unwrap_or_else(|| Scope::new())
        };

        target_scope
    }
}
