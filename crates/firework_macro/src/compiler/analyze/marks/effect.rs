// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use syn::parse::Parser;

pub use super::super::*;

impl Analyzer {
    /// Блок effect!(..., {})
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
                    FireworkAction::ReactiveBlock(FireworkReactiveBlock::Effect, effect_sparks),
                    |this| {
                        for stmt in &last_expr_block.block.stmts {
                            this.visit_stmt(stmt);
                        }
                    }
                );
            } else {
                // [FE012]
                // Эффект должен иметь блок последним аргументом
                self.context.errors.push(
                    compile_error_spanned(
                        i,
                        EFFECT_MISSING_BODY_ERROR,
                    )
                );

                return;
            }
        }
    }
}
