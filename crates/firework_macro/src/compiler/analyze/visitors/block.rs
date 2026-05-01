// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use syn::spanned::Spanned;

pub use super::super::*;

/// Генерация снапшота текущих динамических виджетов в цикле
macro_rules! create_snapshot {
    ($self:expr, $snapshot_name:ident) => {
        $self.context.cycle_depth += 1;
        $self.context.microruntime_widgets.count += 1;
        $self.context.microruntime_widgets.is_dirty = false;

        // Снапшот состояния. Микрорантайм виджеты можно создать только внутри цикла поэтому
        // шума в снапшоте быть не может, после выхода из первого цикла он всегда будет
        // пустым
        let mut $snapshot_name = $self.context.microruntime_widgets.clone();

        // Очистка чтобы у каждой области видимости цикла были только его виджеты
        $self.context.microruntime_widgets.widgets.clear(); 
    };
}

/// Возврат снапшота, требует вызов create_snapshot в той же области видимости выше
macro_rules! restore_snapshot {
    ($self:expr, $snapshot_name:ident) => {
        $snapshot_name.has_widgets = $self.context.microruntime_widgets.has_widgets;
        $self.context.microruntime_widgets = $snapshot_name;

        // Если после замены уровень нулевой то виджетов тут снова нет
        if $self.context.microruntime_widgets.count == 0 {
            $self.context.microruntime_widgets.has_widgets = false;
        }

        if $self.context.cycle_depth > 0 {
            $self.context.cycle_depth -= 1;
        }
    };
}

impl<'ast> Analyzer {
    /// Вход в новую область видимости
    pub(crate) fn analyze_block(&mut self, i: &'ast syn::Block) { 
        // Сначала клонируем всё состояние текущей области видимости, когда эта область
        // видимости закончится (та, что сейчас открывается) все переменные и не только
        // созданные внутри неё будут дропнуты и мы не можем их использовать. После
        // завершения блока нам нужно вернуть ранее сохранённое состояние, а для этого
        // мы будем использовать клон который создаётся здесь
        self.lifetime_manager.old_scope.push(self.lifetime_manager.scope.clone());
        self.lifetime_manager.scope.is_cycle = false;

        // Парсинг области видимости, переменные созданные в этой области видимости будут
        // в self.scrope.variables
        visit::visit_block(self, i); 
 
        // Дебаг вывод всех переменных который собрали в этой новой области видимости
        self.log_scope();

        // Область видимости закончилась, нужно восстановить состояние используя клон
        let scope = self.lifetime_manager.old_scope.pop().unwrap_or(Scope::new()); 

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
                            let else_if_condition_code = else_if.cond.to_token_stream()
                                .to_string();
                        
                            this.handle_reactive_block(
                                else_if_sparks.clone(),
                                false,
                                format!("}} else if {} {{", else_if_condition_code),
                                FireworkAction::ReactiveBlock(
                                    FireworkReactiveBlock::ReactiveIf, else_if_sparks, false,
                                ),
                                |inner_this| {
                                    inner_this.analyze_block(&else_if.then_branch);
                                    
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

                                    else_if.then_branch.brace_token.span
                                },
                            );
                        },

                        syn::Expr::Block(else_block) => { 
                            this.handle_reactive_block(
                                Vec::new(),
                                false,
                                "} else {".to_string(),
                                else_action,
                                |inner_this| {
                                    inner_this.analyze_block(&else_block.block);
                                    else_block.block.brace_token.span
                                },
                            );
                        },

                        _ => {}
                    }
                }

                this.context.is_maybe = old_maybe;
                i.then_branch.brace_token.span
            },
        );
    }

    /// Цикл while
    pub(crate) fn analyze_expr_while(&mut self, i: &'ast ExprWhile) {
        create_snapshot!(self, snapshot);
        
        let sparks = self.get_sparks(&i.cond);
        let condition_code = i.cond.to_token_stream().to_string();

        // Метка цикла
        let label = i.label.as_ref().map(|l| l.name.ident.to_string());

        self.lifetime_manager.scope.is_cycle = true;
        self.lifetime_manager.scope.label = label;
       
        self.handle_reactive_block(
            sparks.clone(),
            true,
            format!("while {} {{", condition_code),
            FireworkAction::ReactiveBlock(FireworkReactiveBlock::ReactiveWhile,
                sparks.clone(), false),
            |this| {
                visit::visit_expr(this, &i.cond);
                this.analyze_block(&i.body);
                i.body.brace_token.span
            }
        );

        self.end_loop(i.span());
        restore_snapshot!(self, snapshot);
    }
    
    /// Цикл for
    pub(crate) fn analyze_expr_for_loop(&mut self, i: &'ast ExprForLoop) {
        create_snapshot!(self, snapshot);
        
        let sparks = self.get_sparks(&i.expr);
        let pattern_code = i.pat.to_token_stream().to_string();
        let expr_code = i.expr.to_token_stream().to_string();

        // Метка цикла
        let label = i.label.as_ref().map(|l| l.name.ident.to_string());

        self.lifetime_manager.scope.is_cycle = true;
        self.lifetime_manager.scope.label = label;
        
        self.handle_reactive_block(
            sparks.clone(),
            true,
            format!("for {} in {} {{", pattern_code, expr_code),
            FireworkAction::ReactiveBlock(FireworkReactiveBlock::ReactiveFor, sparks.clone(),
                false),
            |this| {
                visit::visit_expr(this, &i.expr);
                this.analyze_block(&i.body);
                i.body.brace_token.span
            }
        );

        self.end_loop(i.span());
        restore_snapshot!(self, snapshot);
    }
    
    /// Match
    pub(crate) fn analyze_expr_match(&mut self, i: &'ast ExprMatch) {
        let sparks = self.get_sparks(&i.expr);
        let expr_code = i.expr.to_token_stream().to_string();
        
        self.handle_reactive_block(
            sparks.clone(),
            false,
            format!("match {} {{", expr_code),
            FireworkAction::ReactiveBlock(FireworkReactiveBlock::ReactiveMatch, sparks.clone(),
                false),
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
                        },

                        // variant => 10,
                        _ => {
                            let mut validator = SparkValidator {
                                spark_count: 0,
                                spark_tokens: None,
                                spark_expr: None,
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

    /// Loop { ... }
    pub(crate) fn analyze_expr_loop(&mut self, i: &'ast ExprLoop) {
        create_snapshot!(self, snapshot);

        // Метка цикла
        let label = i.label.as_ref().map(|l| l.name.ident.to_string());

        self.lifetime_manager.scope.is_cycle = true;
        self.lifetime_manager.scope.label = label;

        self.handle_reactive_block(
            Vec::new(),
            true,
            "loop {".to_string(),
            FireworkAction::DefaultCode,
            |this| {
                this.analyze_block(&i.body);
                i.body.brace_token.span
            }
        );

        self.end_loop(i.span());
        restore_snapshot!(self, snapshot);
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
        let target_scope = self.lifetime_manager.item_scope.clone();
        
        self.update_scope(target_scope, false);
        
        visit::visit_expr_return(self, i);
    }

    /// Замыкание
    pub(crate) fn analyze_expr_closure(&mut self, i: &'ast ExprClosure) {
        // Сохранение текущего состояния менеджера времён жизни переменных
        let old_manager = std::mem::replace(&mut self.lifetime_manager, LifetimeManager::new());

        self.lifetime_manager = LifetimeManager::new();
        visit::visit_expr(self, &i.body);
        
        // После анализа делается замена пустого менеджера на старый, это нужно
        // чтобы return в замыканиях не вернул всё реактивные переменные в статику
        // (не сгенерировал DropSpark в замыкании)
        self.lifetime_manager = old_manager;
    }

    fn end_loop(&mut self, span: Span) {
        // Если при выходе из цикла в нём обнаружены виджеты то нужно пометить этот цикл
        // как динамический список в IR
        if self.context.microruntime_widgets.has_widgets {
            let mut statement = self.context.statement.clone();
            
            statement.action = FireworkAction::DynamicLoopBegin(
                self.context.cycle_depth,
                self.context.microruntime_widgets.widgets.clone(),
            );
            statement.string = "".to_string();
            statement.span = span;
            self.context.ir.push(statement); 
        }
    }
}
