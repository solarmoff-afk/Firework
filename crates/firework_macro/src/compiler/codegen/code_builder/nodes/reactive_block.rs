// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use proc_macro2::TokenStream;

use super::super::*;

use crate::compiler::codegen::consts::CHECK_DEC_RB;

impl CodeBuilder {
    /// Реактивный блок это if, for, while, match или эффект (effect!()) который был создан
    /// с хотя-бы одним спарком (реактивной переменной) в условии. Реактивный блок
    /// оборачивается в условие, у каждого спарка есть свой бит в одной из N битовых масок,
    /// он проверяется на активность (1) и если один из битов активен ИЛИ контекст
    /// Build или Navigate то условие проверяется. Благодаря этому достигается мелкозернистая
    /// pull реактивность, блоки сами проверяют изменение своих спарков, а битовые маски
    /// очень лёгкие и быстрые. Если у эффекта нет спарков то он всё равно будет выполнен
    /// при Build или Navigate контексте, если нужен код который выполняется только
    /// при старте или навигации то нужно использовать
    ///
    /// ```ignore
    /// effect!({
    ///     println!("Code...");
    /// });
    /// ```
    ///
    /// Также синтаксис
    /// ```ignore
    /// let mut a = spark!(0);
    /// let mut b = spark!(5);
    ///
    /// a = b * 2;
    /// ```
    ///
    /// Создаст реактивный блок и изменение b обновит a
    ///
    /// Реактивные блоки которые описывают внутри себя UI либо содержат реактивные блоки
    /// которые содержат внутри себя UI срабатывают ещё и при фазе Event, это определяется
    /// по наличию декларации виджета внутри одного из дочерних реактивных блоков
    #[tracing::instrument(skip_all, fields(statements = ?statement))]
    pub fn node_reactive_block(
        &self,
        span: Span,
        final_tokens: &mut TokenStream,
        statement: &FireworkStatement,
        processed_body: &TokenStream,
    ) -> bool {
        if let FireworkAction::ReactiveBlock(_block_type, sparks, is_ui) = &statement.action {
            let mut condition: Vec<TokenStream> = Vec::new();

            // Генерация условия на то, что хотя-бы одна зависимость в снапшотах битовых
            // масках изменилась
            for (_, id) in sparks.iter() {
                let mask_name = get_mask_name(*id);
                let bit_id = normalize_bit_index(*id);

                condition.push(check_flag_tokens(&mask_name, bit_id));
            }

            // Дополнительное условие что контекст Build или Navigate (Более привычный
            // синоним это монтирование). Если реактивный блок является частью декларации
            // UI то ему также нужен и Event чтобы не ломать динамические списки
            let context_check = if *is_ui {
                quote_spanned!(span=> 
                    (::firework_ui::tiny_matches!(_fwc_event, 
                        ::firework_ui::LifeCycle::Navigate | 
                        ::firework_ui::LifeCycle::Build | 
                        ::firework_ui::LifeCycle::Event
                    )) 
                )
            } else {
                quote_spanned!(span=> 
                    (::firework_ui::tiny_matches!(_fwc_event, 
                        ::firework_ui::LifeCycle::Navigate | 
                        ::firework_ui::LifeCycle::Build
                    )) 
                )
            };

            let condition_statement = quote_spanned!(span=>
                #( #condition )||* || #context_check
            );

            final_tokens.extend(quote_spanned!(span=>
                if #condition_statement {
                    #processed_body
                }
            ));

            return true;
        };

        false
    }
}
