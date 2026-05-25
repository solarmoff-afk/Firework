// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub use super::super::*;

impl Analyzer {
    /// Добавляет поле в структуру экрана, если экран ещё не зарегистрирован в FireworkIR
    /// то он создаётся там
    pub fn add_field_to_struct(&mut self, field_name: String, field_type: String) {
        if let Some(_function_name) = &self.function_name {
            // Добавляет значение в вектор (описание структуры экрана или компонента), если
            // такого значения нет в хэш мапе то создаёт пустой вектор
            match self.context.now_component {
                // Вне режима компиляции Component now_component никогда не станет Some,
                // так как он становится таким только в Impl визиторе, а там стоит ранний
                // выход если режим компиляции в флагах не Component
                Some(_) => self.add_field_to_component(field_name, field_type),
                None => self.add_field_to_screen(field_name, field_type),
            }
        }
    }

    fn add_field_to_screen(&mut self, field_name: String, field_type: String) {
        self.context
            .ir
            .screen_structs
            .entry(format!(
                "ApplicationUiBlockStruct{}",
                self.lifetime_manager.scope.screen_index
            ))
            .or_default()
            .push((format!("_fwc_{}", field_name), field_type));
    }

    fn add_field_to_component(&mut self, field_name: String, field_type: String) {
        // SAFETY: Этот метож вызывается только из add_field_to_struct и только если
        // now_component это Some
        self.context
            .ir
            .component_structs
            .entry(self.context.now_component.clone().expect("IE:7"))
            .or_default()
            .push((format!("_fwc_{}", field_name), field_type.clone()));

        self.add_field_to_screen(field_name, field_type);
    }
}
