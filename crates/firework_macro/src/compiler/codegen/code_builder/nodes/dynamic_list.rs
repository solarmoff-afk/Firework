// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::super::*;

impl CodeBuilder {
    pub fn node_dynamic_list(
        &self,
        span: Span,
        final_tokens: &mut TokenStream,
        _struct_name: String,
        statement: &FireworkStatement,
        processed_body: &TokenStream,
    ) -> bool {
        match &statement.action {
            FireworkAction::DynamicLoopBegin(_depth, _widgets) => {
                final_tokens.extend(quote_spanned!(span=>
                    // #list_inits
                    #processed_body
                    // #list_ends
                ));

                return true;
            }

            _ => {}
        };

        false
    }
}
