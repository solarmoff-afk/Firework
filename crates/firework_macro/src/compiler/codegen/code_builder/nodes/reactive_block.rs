// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

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
    pub fn node_reactive_block(
        &self, span: Span, final_tokens: &mut TokenStream, statement: &FireworkStatement,
        processed_body: &TokenStream, all_statements: &[FireworkStatement],
    ) -> bool {
        match &statement.action {
            FireworkAction::ReactiveBlock(_block_type, sparks, is_ui) => {
                let mut condition = String::new();
                
                // Генерация условия на то, что хотя-бы одна зависимость в снапшотах битовых
                // масках изменилась
                for (_, id) in sparks.iter() {
                    self.generate_check_spark_bit(&mut condition, *id);
                    condition.push_str(" ||");
                }
                
                // Дополнительное условие что контекст Build или Navigate (Более привычный
                // синоним это монтирование). Если реактивный блок является частью декларации
                // UI то ему также нужен и Event чтобы не ломать динамические списки
                match is_ui {
                    true => condition.push_str(format!(" {} ", CHECK_DEC_RB).as_str()),
                    false => condition.push_str(format!(" {} ", CHECK_NAVIGATE).as_str()),
                };
                
                let condition_statement = condition.to_expr().unwrap();
                
                final_tokens.extend(quote_spanned!(span=>
                    if #condition_statement {
                        #processed_body
                    }
                ));

                return true;
            },

            _ => {},
        };

        false
    } 
}
