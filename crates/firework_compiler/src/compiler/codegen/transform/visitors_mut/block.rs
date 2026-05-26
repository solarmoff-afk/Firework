// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

#[cfg(feature = "trace")]
use tracing::instrument;

pub use super::super::*;

use crate::compiler::common::widget_kind::is_layout;

impl CodegenVisitor<'_> {
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(
        node = %quote!(#i),
        debug = ?i,
    )))]
    pub(crate) fn analyze_block_mut(&mut self, i: &mut Block) {
        let mut new_statements = Vec::new();
        let original_statements = std::mem::take(&mut i.stmts);

        for mut statement in original_statements {
            let span = statement.span();
            let mut layout_body = None;

            if let Stmt::Macro(m) = &mut statement {
                if let Some(segment) = m.mac.path.segments.last() {
                    let name = segment.ident.to_string();
                    if is_layout(&name) {
                        let tokens = &m.mac.tokens;
                        let tokens_with_braces = quote::quote!({ #tokens });

                        if let Ok(mut inner_block) = syn::parse2::<Block>(tokens_with_braces) {
                            self.analyze_block_mut(&mut inner_block);
                            let processed_children = &inner_block.stmts;
                            layout_body = Some(quote::quote!(#(#processed_children)*));
                        }
                    }
                }
            }

            if layout_body.is_none() {
                syn::visit_mut::visit_stmt_mut(self, &mut statement);
            }

            let body_tokens = layout_body.unwrap_or_else(|| quote::quote!(#statement));

            let ir_statements = {
                if let Some(ir_vec_ref) = self.ir.get_statements_by_span_mut(span) {
                    std::mem::take(ir_vec_ref)
                } else {
                    Vec::new()
                }
            };

            if !ir_statements.is_empty() {
                let generated = self.generate_code(&statement, &ir_statements, body_tokens);
                new_statements.push(Stmt::Expr(Expr::Verbatim(generated), None));
            } else {
                new_statements.push(statement);
            }

            if !ir_statements.is_empty() {
                let ir_vec_ref = self.ir.get_statements_by_span_mut(span).unwrap();
                *ir_vec_ref = ir_statements;
            }
        }
        i.stmts = new_statements;
    }
}
