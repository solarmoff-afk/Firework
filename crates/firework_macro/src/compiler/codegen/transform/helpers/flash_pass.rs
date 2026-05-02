// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::super::*;

impl CodegenVisitor<'_> {
    pub fn generate_flash_pass(&self, id: u128, function_name: &str) -> Block {
        let struct_name_raw = format!("ApplicationUiBlockStruct{}", id);
        let instance_name = struct_name_raw.to_uppercase();

        let mut output = String::new();

        // Чтобы вставить несколько стейтементов нужно использовать Block, а для
        // того чтобы спарсить строку в блок нужно обернуть её в фигурные скобки
        output.push('{');
        output.push_str(&is_first_call(id));

        let fields = self.ir.screen_structs.get(&struct_name_raw);
        output.push_str(&init_instance(
            &instance_name,
            &struct_name_raw,
            fields.unwrap_or(&vec![]),
        ));

        // [FLASH PASS]
        // Flash pass это форма функции или метода которая позволяет использовать
        // одну функцию для нескольких вариантов цикла жизни. Если id экрана не
        // совпадает с айди который сохранён в фреймворке и экран не был построен
        // до этого то это фаза Build, если id не совпадает, но экран был построен
        // то это Navigate, если совпадает то это Event, если была итерация
        // реактивного цикла то Reactive, изначально Zero. Все контексты:
        //  - Build: Первый старт экрана или компонента, инициализируется
        //    состояние. Выполняется только один раз
        //  - Navigate: Переход с одного экрана на другой. Виджеты удаляются
        //    (Как и при navigate) и всё создаётся с нуля
        //  - Event: Какой либо ивент
        //  - Reactive: Пустышка чтобы обновление спарков не запустилось снова без
        //    явной причины. (Детальнее в ../code_builder/nodes/update_spark.rs)
        //  Функция сама устанавливает себя как фокус (SET_FOCUS константа)
        output.push_str(CHECK_EVENT);
        output.push_str(SET_FOCUS);
        output.push_str(&format!("\tfirework_ui::set_focus({});\n", function_name));

        // Код пользователя и реактивный цикл
        output.push('}');

        let flash_pass_block: Block = parse_str(&output).unwrap();
        flash_pass_block
    }
}
