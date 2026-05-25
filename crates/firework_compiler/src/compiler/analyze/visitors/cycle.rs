// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use syn::spanned::Spanned;

pub use super::super::*;

use crate::compiler::analyze::utils::check_expr::ExprAnalyzeResult;

/// Генерация снапшота текущих динамических виджетов в цикле
macro_rules! create_snapshot {
    ($self:expr, $snapshot_name:ident) => {
        $self.context.cycle_depth += 1;
        $self.context.microruntime_widgets.count += 1;
        $self.context.microruntime_widgets.is_dirty = false;

        // Снапшот состояния. Микрорантайм виджеты можно создать только внутри цикла поэтому
        // шума в снапшоте быть не может, после выхода из первого цикла он всегда будет
        // пустым
        let mut $snapshot_name = (
            $self.context.microruntime_widgets.clone(),
            $self.context.first_cycle.clone(),
        );

        // Очистка чтобы у каждой области видимости цикла были только его виджеты
        $self.context.microruntime_widgets.widgets.clear();
    };
}

/// Возврат снапшота, требует вызов create_snapshot в той же области видимости выше
macro_rules! restore_snapshot {
    ($self:expr, $snapshot_name:ident) => {
        $snapshot_name.0.has_widgets = $self.context.microruntime_widgets.has_widgets;
        $self.context.microruntime_widgets = $snapshot_name.0;
        $self.context.first_cycle = $snapshot_name.1;

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
    /// Цикл while
    pub(crate) fn analyze_expr_while(&mut self, i: &'ast ExprWhile) {
        create_snapshot!(self, snapshot);
        self.begin_loop(i.span());

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
            FireworkAction::ReactiveBlock(
                FireworkReactiveBlock::ReactiveWhile,
                sparks.clone(),
                false,
            ),
            |this| {
                visit::visit_expr(this, &i.cond);
                this.analyze_block(&i.body);
                i.body.brace_token.span
            },
        );

        self.end_loop(i.span());
        restore_snapshot!(self, snapshot);
    }

    /// Цикл for
    pub(crate) fn analyze_expr_for_loop(&mut self, i: &'ast ExprForLoop) {
        create_snapshot!(self, snapshot);
        self.begin_loop(i.span());

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
            FireworkAction::ReactiveBlock(
                FireworkReactiveBlock::ReactiveFor,
                sparks.clone(),
                false,
            ),
            |this| {
                visit::visit_expr(this, &i.expr);
                this.analyze_block(&i.body);
                i.body.brace_token.span
            },
        );

        self.end_loop(i.span());
        restore_snapshot!(self, snapshot);
    }

    /// Loop { ... }
    pub(crate) fn analyze_expr_loop(&mut self, i: &'ast ExprLoop) {
        create_snapshot!(self, snapshot);
        self.begin_loop(i.span());

        // Метка цикла
        let label = i.label.as_ref().map(|l| l.name.ident.to_string());

        self.lifetime_manager.scope.is_cycle = true;
        self.lifetime_manager.scope.label = label;

        self.handle_reactive_block(
            ExprAnalyzeResult::new(),
            true,
            "loop {".to_string(),
            FireworkAction::DefaultCode,
            |this| {
                this.analyze_block(&i.body);
                i.body.brace_token.span
            },
        );

        self.end_loop(i.span());
        restore_snapshot!(self, snapshot);
    }

    fn begin_loop(&mut self, span: Span) {
        // Если при выходе из цикла в нём обнаружены виджеты то нужно пометить этот цикл
        // как динамический список в IR
        let mut statement = self.context.statement.clone();

        if self.context.first_cycle.is_none() {
            statement.action = FireworkAction::DynamicLoopBegin(
                self.context.cycle_depth,
                self.context.microruntime_widgets.widgets.clone(),
            );
            statement.string = "".to_string();
            statement.span = span;

            match &self.context.first_ui_reactive_block {
                Some(hook) => {
                    self.context.ir.push_from_key(statement, hook.key.0.clone());
                    self.context.first_cycle = Some(self.get_hook().expect("IE:12"));
                }

                None => self.context.ir.push(statement),
            };

            // Создание хука для первого цикла. Используется is_none так как перезапись
            // может произойти выше
            if let Some(hook) = self.get_hook()
                && self.context.first_cycle.is_none()
            {
                self.context.first_cycle = Some(hook);
            }
        }
    }

    fn end_loop(&mut self, _span: Span) {
        let widgets_clone = self.context.microruntime_widgets.widgets.clone();

        if let Some(hook) = &self.context.first_cycle {
            let statement = self.get_statement_from_hook(hook.clone());
            if let FireworkAction::DynamicLoopBegin(_depth, widgets) = &mut statement.action {
                widgets.extend(widgets_clone);
            }
        }
    }
}
