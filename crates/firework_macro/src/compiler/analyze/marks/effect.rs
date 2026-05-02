// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use syn::parse::Parser;

pub use super::super::*;

impl Analyzer {
    /// Блок effect!(..., {})
    /// Эффект это блок кода который выполняется в двух случаях
    ///  - Фаза флэша Build или Navigate
    ///  - Один из спарков от которых зависит эффект изменился
    ///
    /// Эффекты требуют явно указывать спарки, если их не указать то эффект просто будет
    /// выполнен всегда. Сигнатура effect!(spark1, spark2, {}) или effect!({}), последним
    /// аргументом всегда должен быть блок
    ///
    /// Эффекты выполняются в том порядке в котором объявлены в коде независимо от порядка
    /// обновления спарков что защищает от гонки эффектов, также эффект внутри условия будет
    /// выполнен только если это условие будет верно. Эффект это просто декларация зависимостей
    /// для простого блока Rust кода
    ///
    /// При множестве изменений за один флэш эффект будет выполнен один раз что защищает от
    /// гличей (срабатывании при промежуточных данных в спарке)
    pub(crate) fn effect_marker<'ast>(&mut self, i: &'ast Macro) {
        let parser = punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated;

        if let Ok(punctuated) = parser.parse2(i.tokens.clone()) {
            let mut args: Vec<Expr> = punctuated.into_iter().collect();

            // Последний аргумент должен быть блоком
            if let Some(Expr::Block(last_expr_block)) = args.pop() {
                let mut effect_sparks = Vec::new();

                // Спарки из всех выражений всех аргументов попадают в effect_sparks
                for arg in &args {
                    effect_sparks.append(&mut self.get_sparks(arg));
                }

                // Удаление дубликатов в векторе для оптимизации проверок битовой маски на
                // этапе кодогенерации
                effect_sparks.dedup();

                self.handle_reactive_block(
                    effect_sparks.clone(),
                    false,
                    "{ // effect".to_string(),
                    FireworkAction::ReactiveBlock(
                        FireworkReactiveBlock::Effect,
                        effect_sparks,
                        false,
                    ),
                    |this| {
                        for stmt in &last_expr_block.block.stmts {
                            this.visit_stmt(stmt);
                        }

                        last_expr_block.block.brace_token.span
                    },
                );
            } else {
                // [FE012]
                // Эффект должен иметь блок последним аргументом
                self.context
                    .errors
                    .push(compile_error_spanned(i, EFFECT_MISSING_BODY_ERROR));

                return;
            }
        }
    }
}
