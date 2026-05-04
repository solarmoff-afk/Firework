// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::super::*;

use crate::compiler::codegen::transform::helpers::dynamic_list::generate_lifecycle;

impl CodeBuilder {
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(span = ?span)))]
    pub fn node_dynamic_list(
        &self,
        span: Span,
        final_tokens: &mut TokenStream,
        _struct_name: String,
        statement: &FireworkStatement,
        processed_body: &TokenStream,
    ) -> bool {
        if let FireworkAction::DynamicLoopBegin(_depth, _widgets) = &statement.action {
            let (list_begin, list_end) = generate_lifecycle(&_struct_name, _widgets, span);

            final_tokens.extend(quote_spanned!(span=>
                #list_begin
                
                #processed_body
                
                #list_end
            ));

            return true;
        };

        false
    }
}
