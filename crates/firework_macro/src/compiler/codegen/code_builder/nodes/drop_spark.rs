// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::super::*;

impl CodeBuilder {
    /// Эта нода нужна чтобы вернуть владение над данными в спарк переменной обратно в
    /// статическую память. Когда используется маркер spark!(0) компилятор генерирует
    /// код для того чтобы инициализировать переменную в статике (Some(0), где 0 это
    /// оригинальное выражение внутри маркера) в контексте Build после чего в любом
    /// контексте забирает владение из статики на стэк. Владение у пользователя, теперь
    /// анализатор начинает отслеживать время жизни переменной и когда она должна умереть
    /// (Выйдя из области видимости, RAII) то в IR добавляется DropSpark. Он нужен
    /// чтобы вернуть владение обратно в статику чтобы не было паники при использовании
    /// take в следующем флэше
    #[tracing::instrument(skip_all, fields(span = ?_span))]
    pub fn node_drop_spark(
        &self,
        _span: Span,
        struct_name: String,
        final_tokens: &mut TokenStream,
        statement: &FireworkStatement,
    ) -> bool {
        // Генерация возврата владения в BSS
        // TODO: Могут возникнуть ошибки компиляции на уровне rustc если
        // пользователь переместит владение, так как возврат владения сделать
        // будет нельзя (Ибо rustc проверит владение на этой строке). Нужно
        // добавить магию компилятора в будущем
        if let FireworkAction::DropSpark { name, id } = &statement.action {
            let field_name = format!("spark_{}", id);
            let set_field_str = static_gen::set_field(&struct_name, &field_name, name);
            let set_field_expr = Self::convert_string_to_syn(&set_field_str);

            final_tokens.extend(quote!(
                #set_field_expr
            ));

            return true;
        }

        false
    }
}
