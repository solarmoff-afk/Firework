// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::super::*;

impl CodeBuilder {
    /// SparkRef это специальный маркер который работает внутри shared! {}, он нужен чтобы
    /// взять ссылку на поле состояния. Если состояние объявлено в сегменте state! {} внутри
    /// shared! {} то его можно получить через spark_ref, если записать ссылку на него в
    /// мутабельную переменную (с mut) то ссылка будет мутабельная, если без mut то
    /// немутабельной. Также изменение в одной функции не выполнит логику в другой
    /// автоматически, реактивность работает только в каждой функции локально, но у функций
    /// есть общее хранилище данных
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

