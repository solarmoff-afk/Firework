// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use syn::spanned::Spanned;

pub use super::super::*;

use crate::compiler::analyze::utils::check_expr::ExprAnalyzeResult;

impl<'ast> Analyzer {
    /// Вход в новую область видимости
    pub(crate) fn analyze_block(&mut self, i: &'ast syn::Block) {
        // Сначала клонируем всё состояние текущей области видимости, когда эта область
        // видимости закончится (та, что сейчас открывается) все переменные и не только
        // созданные внутри неё будут дропнуты и мы не можем их использовать. После
        // завершения блока нам нужно вернуть ранее сохранённое состояние, а для этого
        // мы будем использовать клон который создаётся здесь
        self.lifetime_manager
            .old_scope
            .push(self.lifetime_manager.scope.clone());
        self.lifetime_manager.scope.is_cycle = false;

        // Парсинг области видимости, переменные созданные в этой области видимости будут
        // в self.scrope.variables
        visit::visit_block(self, i);

        // Дебаг вывод всех переменных который собрали в этой новой области видимости
        self.log_scope();

        // Область видимости закончилась, нужно восстановить состояние используя клон
        let scope = self
            .lifetime_manager
            .old_scope
            .pop()
            .unwrap_or(Scope::new());

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
        if !sparks.len() > 0 {
            else_action = FireworkAction::ReactiveElse;
        }

        // Исходная строка созданная из токенов внутри условия блока
        let condition_code = i.cond.to_token_stream().to_string();

        self.handle_reactive_block(
            sparks.clone(),
            false,
            format!("if {} {{", condition_code),
            FireworkAction::ReactiveBlock(FireworkReactiveBlock::ReactiveIf, sparks, false),
            |this| {
                let old_maybe = this.context.is_maybe;
                this.context.is_maybe = true;

                // Основное тело условия
                this.analyze_block(&i.then_branch);

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
                                    FireworkReactiveBlock::ReactiveIf,
                                    else_if_sparks,
                                    false,
                                ),
                                |inner_this| {
                                    inner_this.analyze_block(&else_if.then_branch);

                                    // Вложенные else/else if
                                    if let Some((_, inner_else_branch)) = &else_if.else_branch {
                                        match &**inner_else_branch {
                                            Expr::If(inner_else_if) => {
                                                inner_this.analyze_expr_if(inner_else_if);
                                            }

                                            syn::Expr::Block(inner_block) => {
                                                inner_this.analyze_block(&inner_block.block);
                                            }

                                            _ => {}
                                        }
                                    }

                                    else_if.then_branch.brace_token.span
                                },
                            );
                        }

                        syn::Expr::Block(else_block) => {
                            this.handle_reactive_block(
                                ExprAnalyzeResult::new(),
                                false,
                                "} else {".to_string(),
                                else_action,
                                |inner_this| {
                                    inner_this.analyze_block(&else_block.block);
                                    else_block.block.brace_token.span
                                },
                            );
                        }

                        _ => {}
                    }
                }

                this.context.is_maybe = old_maybe;
                i.then_branch.brace_token.span
            },
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
            FireworkAction::ReactiveBlock(
                FireworkReactiveBlock::ReactiveMatch,
                sparks.clone(),
                false,
            ),
            |this| {
                let old_maybe = this.context.is_maybe;
                this.context.is_maybe = true;

                for arm in &i.arms {
                    if let Some((_, guard)) = &arm.guard {
                        visit::visit_expr(this, guard);
                    }

                    // Анализируем тело ветки
                    match &*arm.body {
                        // Блок variant => { ... },
                        Expr::Block(expr_block) => {
                            this.analyze_block(&expr_block.block);
                        }

                        // variant => 10,
                        _ => {
                            let mut validator = SparkValidator {
                                spark_count: 0,
                                spark_tokens: None,
                                spark_expr: None,
                                spark_async_closure: None,
                                spark_parse_error: None,
                            };
                            validator.visit_expr(&arm.body);

                            if validator.spark_count > 0 {
                                // [FE014]
                                // Спарки нельзя создавать условно в матче
                                this.context.errors.push(compile_error_spanned(
                                    &arm.body,
                                    SPARK_BLOCK_REQUIRED_ERROR,
                                ));
                            } else {
                                visit::visit_expr(this, &arm.body);
                            }
                        }
                    }
                }

                this.context.is_maybe = old_maybe;
                i.brace_token.span
            },
        );
    }
}
