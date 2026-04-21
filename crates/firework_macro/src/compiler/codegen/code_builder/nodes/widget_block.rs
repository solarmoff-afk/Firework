// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::super::*;

impl CodeBuilder { 
    pub fn node_widget_block(
        &self,
        span: Span,
        struct_name: String,
        final_tokens: &mut TokenStream,
        statement: &FireworkStatement, 
    ) {
        match &statement.action {
            FireworkAction::WidgetBlock(widget_type, fields, is_functional, id, has_microruntime) => {
                final_tokens.extend(quote_spanned!(span=> 
                    println!("Widget");
                ));
            },

            _ => {},
        };
    }
}


