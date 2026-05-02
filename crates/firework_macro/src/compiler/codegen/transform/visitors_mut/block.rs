// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use quote::quote;

pub use super::super::*;

impl CodegenVisitor<'_> {
    pub(crate) fn analyze_block_mut(&mut self, i: &mut Block) {
        let mut new_statements = Vec::new();

        let original_statements = std::mem::take(&mut i.stmts);

        for mut statement in original_statements {
            let span = statement.span();
            let ir_statements = self.ir.get_statements_by_span(span).cloned();

            let mut body_statements = Vec::new();

            syn::visit_mut::visit_stmt_mut(self, &mut statement);
            body_statements.push(statement.clone());

            let body_tokens = quote!(
                #(#body_statements)*
            );

            if let Some(ir_list) = ir_statements {
                let generated_tokens = self.generate_code(&statement, &ir_list, body_tokens);
                let wrapper_block: Block = parse_quote!(
                    {
                        #generated_tokens
                    }
                );

                new_statements.extend(wrapper_block.stmts);
            } else {
                new_statements.extend(body_statements);
            }
        }

        i.stmts = new_statements;
    }
}
