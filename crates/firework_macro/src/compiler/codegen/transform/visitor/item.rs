// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub use super::super::*;

impl CodegenVisitor<'_> {
    /// Обрабатывает верхний уровень в вызове компилятора (item), функции, структуры и так
    /// далее. Генерирует flash pass и реактивный цикл
    pub(crate) fn analyze_file_mut(&mut self, i: &mut File) {
        let mut new_items = Vec::new();
        
        // Забираем элементы, чтобы не клонировать весь вектор сразу
        let items = std::mem::take(&mut i.items);

        for item in items {
            // Любая функция это экран
            if let Item::Fn(mut item_fn) = item {
                let function_name = item_fn.sig.ident.to_string();
                
                // Поиск имени функции в IR, если оно не найдено от find вернёт None в self.ui_id
                // и код для UI не сгенерируется
                self.ui_id = self.ir.screens
                    .iter()
                    .find(|(name, _, _)| name == &function_name)
                    .map(|(_, _, id)| *id);

                visit_item_fn_mut(self, &mut item_fn);

                if let Some(id) = self.ui_id {
                    let span = item_fn.span();

                    // Имя структуры экрана, сырое для генератора и имя для вставки через
                    // quote
                    let struct_name_raw = format!("ApplicationUiBlockStruct{}", id);
                    let struct_name = format_ident!("ApplicationUiBlockStruct{}", id);

                    // Вектор полей структуры, хранит кортежи (имя, тип). Они собраны
                    // анализатором для имени структуры ApplicationUiBlockStruct{id}
                    let default = Vec::new();
                    let fields_data = self.ir.screen_structs
                        .get(&format!("ApplicationUiBlockStruct{}", id))
                        .unwrap_or(&default);

                    // Представление полей для вставки
                    let mut fields: Vec<Field> = Vec::new(); 

                    // Проход по всем сырым полям чтобы сгенерировать field через quote 
                    // с сохранением спана (для ошибок)
                    for (field_name_raw, field_type_raw) in fields_data {
                        // Имя и тип поля
                        let field_name = format_ident!("{}", field_name_raw);
                        let field_type: Type = parse_str(field_type_raw).unwrap();
                        
                        // Кодогенерация поля
                        let field = parse_quote_spanned!(span=> 
                            #field_name: core::option::Option<#field_type>
                        );
                        
                        fields.push(field);
                    }

                    // Генерация статического экземпляра. Если используется safety-multitrhead
                    // фича то static_gen генерирует OnceLock + Mutex для безопасной работы
                    // из нескольких поток, если safety-multitrhead нет то используется
                    // static mut и unsafe
                    let instance_name = struct_name_raw.to_uppercase();
                    let instance = static_declaration(&instance_name, &struct_name_raw, fields_data);
                    let instance_item: Item = parse_str(&instance).unwrap();

                    // Структура экрана где хранится состояние, компоненты и виджеты
                    let struct_def: Item = parse_quote_spanned!(span=> 
                        struct #struct_name {
                            #(#fields),*
                        }
                    );
                    
                    new_items.push(struct_def);
                    new_items.push(instance_item);
                    
                    // Оригинальное тело функции (уже трансформированное), так как block
                    // не реализует Default нужно использовать std::mem::replace, идёт
                    // парсинг обычного пустого блока чтобы заменить на него оригинал, а
                    // оригинальые данные забрать сюда чтобы избежать клонирования
                    let original_block = std::mem::replace(&mut item_fn.block, parse_quote!({}));

                    let flash_pass_block = self.generate_flash_pass(id, &function_name);
                    let reactive_generated = self.generate_reactive(id);

                    let bitmask_statements = reactive_generated.bitmask_statements;
                    let bitmask_clone_statements = reactive_generated.bitmask_clone_statements;
                    let bitmask_check_expr = reactive_generated.bitmask_check_expr;

                    // Если цикл совершил более 64 итераций (хардкод )то происходит выход
                    // из него это делается после добавления единицы к итерациям чтобы не
                    // отнимать единицу
                    // (64 - 1 = 63) от максимального количества итераций, так как:
                    //  - Нулевой шаг, +1, 1 итерация
                    //  - Первый шаг,  +1, 2 итерация
                    //  - 63 шаг, +1,  +1, 64 итерация, условие сработало 
                    item_fn.block = parse_quote_spanned!(span=> {
                        let mut _fwc_build = false;
                        let mut _fwc_event = firework_ui::LifeCycle::Zero;

                        #flash_pass_block

                        let mut _fwc_guard: u8 = 0;
                        #(#bitmask_statements)*

                        loop {
                            #(#bitmask_clone_statements)*
                            #original_block

                            if #bitmask_check_expr { break; }
                            _fwc_guard += 1;
                            _fwc_event = firework_ui::LifeCycle::Reactive;
                            if _fwc_guard > 64 { break; }
                        }
                    });
                }
                new_items.push(Item::Fn(item_fn));
            } else {
                let mut other_item = item;

                syn::visit_mut::visit_item_mut(self, &mut other_item);
                
                new_items.push(other_item);
            }
        }
        
        i.items = new_items;
    }
}
