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
            FireworkAction::SparkRef { name, id, is_mut, root, local_id, .. } => {
                let field_name = format!("spark_{}", id);
               
                // Если переменная для записи мутабельная то и ссылка нужна мутабельная,
                // если нет то немутабельная
                let ref_field_str = match *is_mut {
                    true => static_gen::get_field_ref_mut(&struct_name, &field_name, &name),
                    false => static_gen::get_field_ref(&struct_name, &field_name, &name),
                };
                
                let ref_field_expr = Self::convert_string_to_syn(&ref_field_str);

                // Условие на изменение спарк ссылки в битовой маске. При изменении ссылки
                // цикл сделает итерацию так как выход не сработает и в начале это условие
                // выполнится, тем самым запустятся функциональные эффекты
                let mut condition = String::new();
                self.generate_check_spark_bit(&mut condition, *local_id);

                // SAFETY: Паники не будет так как generate_check_spark_bit всегда
                // генерирует валидный код, а к результату не добавляется никакой
                // пользовательский код
                let condition_statement = condition.to_expr().unwrap();

                // Сбор функциональных эффектов для этого разделяемого состояния.
                // Функциональные эффекты позволяют писать отдельные функции в shared
                // блоке которые будут выполнятся при любом изменении разделяемого состояния
                // в shared блоке. Обычные эффекты работают только на уровне функции, из
                // другой функции Б изменение состояния не запустит локальный эффект в функции А
                let temp = Vec::new();
                let func_effects = self.ir.shared.effects.get(root).unwrap_or(&temp);
                let mut func_effects_statements = Vec::new();

                // Проход по всем функциональным эффектам которые привязаны к этому 
                // функциональному состоянию и генерация вызовов
                for effect in func_effects {
                    let ident = format_ident!("{}", effect);
                    func_effects_statements.push(quote! {
                        #ident();
                    })
                }

                // Генерация ссылки и проверки на изменение чтобы вызвать функциональные
                // эффекты
                final_tokens.extend(quote_spanned!(span=> 
                    #ref_field_expr

                    if #condition_statement {
                        #(#func_effects_statements)*
                    }
                ));
            },

            _ => {},
        };
    }
}
