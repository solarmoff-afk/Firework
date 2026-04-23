// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::super::*;

impl CodeBuilder {
    /// Обновление реактивной переменной. Оно работает без .set/.get, вместо него
    /// аналазитор отслеживает name = 10, name += 1, name.field = 1 и name.mut_fn()
    /// Спарк это не обёртка или умный указатель, а просто T который реактивный
    /// благодаря магии компилятора
    pub fn node_update_spark(
        &self, span: Span, final_tokens: &mut TokenStream,
        statement: &FireworkStatement, processed_body: &TokenStream,
    ) {
        match &statement.action {
            FireworkAction::UpdateSpark(_, id, _) => {
                // Реактивная переменная (спарк) обновилась то нужно изменить бит
                // который соотвествует этому спарку. Для каждого диапазона спарков
                // (от 0 до 64) своя битовая маска, поэтому эта строка позволяет
                // определить в какой маске изменить спарк
                let mask = get_spark_mask(*id);

                // Если это обновление спарка которое не находится в реактивном
                // блоке либо в ивенте то оно должно сработать только при Navigate
                // или Build для того чтобы не было сюрпризов. При следующем шаге
                // цикла (если хотя-бы один бит в одной из битовых масок активен то
                // break не выполнится) _fwc_event изменит своё значение со старого
                // на LifeCycle::Reactive из-за чего это не позволит создать цикл
                // на 64 итерации (или другое ограничение _fwc_guard) при подобном
                // коде
                //
                // fn test_screen() {
                //  // Spark...
                //
                //  // Это сработает только если _fwc_event это Build или Navigate,
                //  // как только значение изменится и в битовой маске появится
                //  // активный бит то _fwc_event станет LifeCycle::Reactive 
                //  // из-за чего этот код не будет выполнен и не начнёт обновлять
                //  // битовую маску каждую итерацию
                //  spark1 += 1;
                // }
                let need_condition = !statement.is_reactive_block
                    && statement.parent_widget_id.is_none();
 
                let statement = format!("{};", set_flag(
                    format!("_fwc_bitmask{}", mask).as_str(),
                    
                    // Используется айди спарка как бит для отслеживания, но
                    // перед этим он проходит через нормализацию (id % 64)
                    // который позволяет использовать даже айди больше 64 
                    // для множества битовых масок
                    normalize_bit_index(*id),
                )).to_stmt().unwrap();

                if need_condition {
                    final_tokens.extend(quote_spanned!(span=>
                        if firework_ui::tiny_matches!(_fwc_event,
                                firework_ui::LifeCycle::Navigate) ||
                            firework_ui::tiny_matches!(_fwc_event,
                                firework_ui::LifeCycle::Build) {
                            
                            #statement
                            #processed_body
                        }
                    ));
                }
            },

            _ => {},
        };
    }
}
