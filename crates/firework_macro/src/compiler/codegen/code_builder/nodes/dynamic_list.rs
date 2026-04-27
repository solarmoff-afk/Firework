// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::super::*;

impl CodeBuilder { 
    pub fn node_dynamic_list(
        &self, span: Span, final_tokens: &mut TokenStream,
        statement: &FireworkStatement, processed_body: &TokenStream,
    ) -> bool {
        match &statement.action {
            FireworkAction::DynamicLoopBegin(depth, widgets) => {
                println!("Depth: {depth}, widgets: {:#?}", widgets);
                final_tokens.extend(quote_spanned!(span=> 
                    println!("Cycle start: {}", #depth);
                    #processed_body
                    println!("Cycle end: {}", #depth);
                ));

                return true;
            },

            _ => {},
        };
        
        false
    }
}
