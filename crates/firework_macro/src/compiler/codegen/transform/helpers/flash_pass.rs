// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use quote::quote;

use super::super::*;

impl CodegenVisitor<'_> {
    #[tracing::instrument(skip_all, fields(function_name = ?function_name))]
    pub fn generate_flash_pass(&self, id: u128, function_name: &str) -> Block {
        let struct_name_raw = format!("ApplicationUiBlockStruct{}", id);
        let instance_name = struct_name_raw.to_uppercase();

        let fields = self.ir.screen_structs.get(&struct_name_raw)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        let instance_init = init_instance_tokens(&instance_name, &struct_name_raw, fields);
        let fn_path: Path = syn::parse_str(function_name)
            .unwrap_or_else(|_| panic!("Invalid function path: {}", function_name));

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
        let block_tokens = quote! {
            {
                let _fwc_id: u128 = #id;

                #instance_init

                if _fwc_id == ::firework_ui::get_focus_id() && !_fwc_build {
                    _fwc_event = ::firework_ui::LifeCycle::Event;
                } else {
                    if _fwc_build {
                        ::firework_ui::adapter_command(::firework_ui::AdapterCommand::RemoveAll);
                        _fwc_event = ::firework_ui::LifeCycle::Build;
                    } else {
                        ::firework_ui::adapter_command(::firework_ui::AdapterCommand::RemoveAll);
                        _fwc_event = ::firework_ui::LifeCycle::Navigate;
                    }
                }

                ::firework_ui::set_focus_id(_fwc_id);
                ::firework_ui::set_focus(#fn_path);
            }
        };

        syn::parse2(block_tokens).expect("IE: generate_flash_pass assembly failed")
    }
}
