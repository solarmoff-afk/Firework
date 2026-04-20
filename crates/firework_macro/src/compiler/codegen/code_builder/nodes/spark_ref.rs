// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::super::*;

impl CodeBuilder {
    pub fn node_spark_ref(
        &self,
        span: Span,
        struct_name: String,
        final_tokens: &mut TokenStream,
        statement: &FireworkStatement,
    ) {
        match &statement.action {
            FireworkAction::SparkRef { name, id, is_mut, .. } => {
                let field_name = format!("spark_{}", id);
                
                let mut ref_field_str = static_gen::get_field_ref(&struct_name, &field_name, &name);
                if *is_mut {
                    ref_field_str = static_gen::get_field_ref_mut(&struct_name, &field_name, &name);
                }
                
                let ref_field_expr = Self::convert_string_to_syn(&ref_field_str);

                final_tokens.extend(quote_spanned!(span=> 
                    #ref_field_expr
                ));
            },

            _ => {},
        };
    }
}

