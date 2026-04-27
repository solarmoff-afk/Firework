// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::super::*;

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
    pub fn node_reactive_block(
        &self, span: Span, final_tokens: &mut TokenStream, statement: &FireworkStatement,
        processed_body: &TokenStream, all_statements: &[FireworkStatement],
    ) -> bool {
        match &statement.action {
            FireworkAction::ReactiveBlock(_block_type, sparks) => {
                let mut condition = String::new();
                
                // Генерация условия на то, что хотя-бы одна зависимость в снапшотах битовых
                // масках изменилась
                for (_, id) in sparks.iter() {
                    self.generate_check_spark_bit(&mut condition, *id);
                    condition.push_str(" ||");
                }
                
                // Дополнительное условие что контекст Build или Navigate (Более привычный
                // синоним это монтирование)
                condition.push_str(format!(" {} ", CHECK_NAVIGATE).as_str());
                
                let condition_statement = condition.to_expr().unwrap();

                // Если внутри реактивного блока происходит обновление спарка то его нужно
                // обработать прямо здесь и обновить биты в битовых масках чтобы перезапустить
                // реактивный цикл
                let mut inner_masks = TokenStream::new();
                for stmt in all_statements {
                    if let FireworkAction::UpdateSpark(_, id, _) = &stmt.action {
                        let mask = get_spark_mask(*id);

                        let update_stmt = format!("{};", set_flag(
                            format!("_fwc_bitmask{}", mask).as_str(),
                            normalize_bit_index(*id),
                        )).to_stmt().unwrap();

                        // Обновлене условных виджетов декларация которых зависит от
                        // этого спарка
                        let update_widgets_statement = self.generate_widget_spark_update(
                            statement, id
                        );
                        
                        inner_masks.extend(quote_spanned!(span=> {
                            #update_widgets_statement
                            #update_stmt
                        }));
                    }
                }
                
                final_tokens.extend(quote_spanned!(span=>
                    if #condition_statement {
                        #processed_body
                        #inner_masks
                    }
                ));

                return true;
            },

            _ => {},
        };

        false
    } 
}
