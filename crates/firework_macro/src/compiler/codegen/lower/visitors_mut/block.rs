// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::super::macro_resolver::MacroResolver;
use super::*;

use crate::compiler::codegen::ir::FireworkAction;

impl LowerVisitor<'_> {
    pub(crate) fn lower_block_mut(&mut self, i: &mut Block) {
        let mut new_statements = Vec::new();
        let original_statements = i.stmts.clone();
        i.stmts.clear();

        for mut statement in original_statements {
            if let Some(expanded_statements) = MacroResolver::expand(&statement) {
                let mut inner_block = Block {
                    brace_token: Default::default(),
                    stmts: expanded_statements,
                };
                self.lower_block_mut(&mut inner_block);
                new_statements.extend(inner_block.stmts);
            } else {
                self.visit_stmt_mut(&mut statement);
                new_statements.push(statement);
            }
        }

        let closing_span = i.brace_token.span.close();

        if let Some(ir_list) = self.ir.get_statements_by_span(closing_span).cloned() {
            let struct_name = format!("ApplicationUiBlockStruct{}", self.ui_id.unwrap_or(0));
            let mut drop_tokens = TokenStream::new();

            for stmt in ir_list {
                if let FireworkAction::DropSpark { .. } = stmt.action {
                    self.builder.node_drop_spark(
                        closing_span,
                        struct_name.clone(),
                        &mut drop_tokens,
                        &stmt,
                    );
                }
            }

            if !drop_tokens.is_empty() {
                self.pending_drops.push((closing_span, drop_tokens));
            }
        }

        i.stmts = new_statements;
    }
}
