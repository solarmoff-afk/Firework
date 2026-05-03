// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use quote::quote;

#[cfg(feature = "trace")]
use tracing::instrument;

pub use super::super::*;

impl CodegenVisitor<'_> {
    #[instrument(skip_all, fields(node = %quote!(#i)))]
    pub(crate) fn analyze_block_mut(&mut self, i: &mut Block) {
        let mut new_statements = Vec::new();

        let original_statements = std::mem::take(&mut i.stmts);

        for mut statement in original_statements {
            let span = statement.span();
            
            let ir_statements = {
                if let Some(ir_vec_ref) = self.ir.get_statements_by_span_mut(span) {
                    std::mem::take(ir_vec_ref)
                } else {
                    Vec::new()
                }
            };

            let mut body_statements = Vec::new();

            syn::visit_mut::visit_stmt_mut(self, &mut statement);
            body_statements.push(statement.clone());

            let body_tokens = quote!(
                #(#body_statements)*
            );

            if !ir_statements.is_empty() {
                let generated_tokens = self.generate_code(&statement, &ir_statements, body_tokens);

                new_statements.push(Stmt::Expr(
                    Expr::Verbatim(generated_tokens),
                    None
                ));
            } else {
                new_statements.extend(body_statements);
            }

            if !ir_statements.is_empty() {
                let ir_vec_ref = self.ir.get_statements_by_span_mut(span).unwrap();
                *ir_vec_ref = ir_statements;
            }
        }

        i.stmts = new_statements;
    }
}
