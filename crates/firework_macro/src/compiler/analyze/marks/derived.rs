// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use proc_macro2::Group;
use syn::spanned::Spanned;

pub use super::super::*;

impl<'ast> Analyzer {
    /// Проверяет обёрнуто ли выражение в derived!() маркер
    pub(crate) fn is_derived_macro(&self, expr: &Expr) -> bool {
        matches!(expr, Expr::Macro(macro_expr) if macro_expr.mac.path.is_ident("derived"))
    }

    /// Получить внутреннее выражение из derived!()
    pub(crate) fn extract_derived_inner(&self, expr: &Expr) -> Option<Expr> {
        match expr {
            Expr::Macro(macro_expr) if macro_expr.mac.path.is_ident("derived") => {
                syn::parse2::<Expr>(macro_expr.mac.tokens.clone()).ok()
            }

            _ => None,
        }
    }

    /// Этот метод реализует поиск спарков в правой части присваивания к реактивной переменной
    /// и если выражение обёрнуто в derived!(), то оборачивает весь statement в эффект, который
    /// подписан на все спарки, используемые в выражении. Позволяет писать spark1 = derived!(spark2 + spark3)
    /// более легко (без effect!(..., {})) и делать код интуитивно понятным. Второй аргумент это
    /// statement который будет вставлен в IR как внутрянка эффекта
    pub(crate) fn compute_derived_spark(
        &mut self,
        right: &'ast Expr,
        mut statement: FireworkStatement,
        root: (&String, usize),
    ) {
        if !self.is_derived_macro(right) {
            // При отсутствии derived!() происходит просто пуш в IR
            self.context.ir.push(statement);
            return;
        }

        let Some(inner_expr) = self.extract_derived_inner(right) else {
            self.context.ir.push(statement);
            return;
        };

        let mut effect_sparks = self.get_sparks(&inner_expr);

        // Если в спарках есть корневая переменная то удаление из вектора, эффект не
        // будет создан если спарков не будет в выражении
        effect_sparks.delete_spark(root);

        let span = inner_expr.span();
        let tokens = inner_expr.to_token_stream();

        for (_, id) in effect_sparks.sparks.iter() {
            self.linter.depend_spark(root.1, *id, tokens.span());
        }

        if !effect_sparks.is_empty() {
            // HACK: Эффекты от вычислительных спарков должны также проходить через
            // handle_reactive_block, но замыкание должно возвращать DelimSpan который
            // нельзя создать, но можно получить из группы. Здесь создаётся группа со
            // спаном из правой части производного выражения и из неё получается delim_span
            let mut dummy_group = Group::new(proc_macro2::Delimiter::Brace, TokenStream::new());

            dummy_group.set_span(span);
            let delim_span = dummy_group.delim_span();

            self.handle_reactive_block(
                effect_sparks.clone(),
                false,
                "{ // effect".to_string(),
                FireworkAction::ReactiveBlock(FireworkReactiveBlock::Effect, effect_sparks, false),
                |this| {
                    // Так как условие if effect_sparks.len() > 0 { выше не сработало бы
                    // и этот код не выполнился бы если в выражении нет спарков то блок
                    // здесь точно реактивный. Это не затронет self.statement так как
                    // statement это клон
                    statement.is_reactive_block = true;

                    this.context.ir.push(statement);
                    delim_span
                },
            );

            return;
        }

        // При отсутствии спарков в выражении происходит немедленный пуш в IR
        self.context.ir.push(statement);
    }
}
