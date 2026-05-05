// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::super::*;

use crate::CompileType;

impl CodeBuilder {
    /// Инициализация реактивной переменной. Сюда вписывается первое значение которое
    /// будет у спарка при инициализаци. Оно будет установлено в статику при первом
    /// запуске экрана (контекст Build) и дальше будет происходить аренда на стэк
    /// через .take из статики. Инициализировать спарк можно через let mut a = spark!(0)
    /// (или другое значение, часто может быть нужен тип данных) либо через
    ///
    /// ```ignore
    /// let mut my_spark: u32 = spark!({
    ///     println!("Код");
    ///     10
    /// });
    /// ```
    ///
    /// Тип во второй записи нужно указывать всегда. Код внутри будет выполнен только
    /// один раз за жизнь экрана (Только при контексте Build), тип данных нужно указывать
    /// так как значение поднимается в статику и является полем структуры после компиляции,
    /// но анализатор иногда может угадать тип если он очевиден. Спарки без mut менять нельзя
    /// что позволяет создать данные которые защищены от изменения, но при этом которые
    /// не пересчитываются каждый раз
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(span = ?span)))]
    pub fn node_initial_spark(
        &self,
        span: Span,
        struct_name: String,
        final_tokens: &mut TokenStream,
        statement: &FireworkStatement,
    ) -> bool {
        if let FireworkAction::InitialSpark {
            id,
            expr_body,
            expr_body_tokens,
            name,
            is_mut,
            ..
        } = &statement.action
        {
            let field_name = format!("spark_{}", id);
            let struct_name_upper = struct_name.to_uppercase();

            // Если спарк инициализирован как мутабельный то нужно создать мутабельную
            // переменную для аренды из статики. Если спарк был инициализирован без mut
            // то переменная создаётся также без mut чтобы не было предупреждения.
            // Ключевая хитрость в том что вместо
            //  let mut a = spark!(0);
            // Создаётся примерно
            //  let mut a = {instance_name}.spark0.take();
            // Владение полностью у пользователя, никаких обёрток, ссылок и уиных
            // указателей
            let mut modifier = TokenStream::new();
            if *is_mut {
                modifier = quote!(mut);
            }

            let ident = format_ident!("{}", name);
            let field_name_ident = format_ident!("{}", field_name);

            match self.flags.compile_type {
                CompileType::Component => {
                    final_tokens.extend(quote_spanned!(span=>
                        if firework_ui::tiny_matches!(
                            _fwc_event,
                            firework_ui::LifeCycle::Build
                        ) {
                            self.#field_name_ident = Some(#expr_body_tokens);
                        }

                        let #modifier #ident = self.#ident.expect("State not init").take();
                    ));
                }

                _ => {
                    let set_field_str =
                        static_gen::set_field(&struct_name, &field_name, &expr_body.to_string());

                    let set_field_expr = Self::convert_string_to_syn(&set_field_str);

                    let take_field_str = static_gen::take_field(&struct_name_upper, &field_name);

                    let take_field_expr = Self::convert_string_to_syn(&take_field_str);

                    final_tokens.extend(quote_spanned!(span=>
                        if firework_ui::tiny_matches!(
                            _fwc_event,
                            firework_ui::LifeCycle::Build
                        ) {
                            #set_field_expr
                        }

                        let #modifier #ident = #take_field_expr;
                    ));
                }
            }

            return true;
        };

        false
    }
}
