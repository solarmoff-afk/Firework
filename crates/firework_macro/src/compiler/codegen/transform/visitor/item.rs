// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub use super::super::*;

use super::super::traits::ToTokenStreams;

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
                    let screen_name = &function_name;
                    
                    let mut output = String::new();
                    
                    // Чтобы вставить несколько стейтементов нужно использовать Block, а для
                    // того чтобы спарсить строку в блок нужно обернуть её в фигурные скобки
                    output.push('{');
                    output.push_str(&is_first_call(id)); 
                    
                    if let Some(fields) = self.ir.screen_structs.get(&struct_name_raw) {
                        output.push_str(&init_instance(&instance_name, &struct_name_raw, fields));
                    } else {
                        output.push_str(&init_instance(&instance_name, &struct_name_raw, &[]));
                    }
                    
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
                    output.push_str(CHECK_EVENT);
                    output.push_str(SET_FOCUS);
                    output.push_str(&format!("\tfirework_ui::set_focus({});\n", screen_name));
                    
                    // Код пользователя и реактивный цикл
                    output.push_str("\n\t// Phase 2: Navigate/Build code\n");
                    output.push('}');

                    // [REACTIVE_LOOP]
                    // Реализация реактивного цикла: Реактивный цикл это loop в который
                    // оборачивается пользовательский код второй фазы. Для каждых 64 
                    // спарков создаётся битовая маска типа u64
                    //
                    // > Выход из цикла
                    //  Выход из цикла происходит в результате одного из двух условий
                    //   - Цикл сделал больше 64 итераций. Используется специальный счётчик
                    //     _fwc_guard к которому добавляется 1 при каждой итерации цикла.
                    //     Если он больше 64 то цикл нужно завершить чтобы не позволить
                    //     создание циклических зависимостей (a -> b; b -> a)
                    //   - Стабильность масок. Если во всех битовых масках все биты
                    //     деактивированы (ноль) то тогда происходит выход из цикла так
                    //     как обновлений реактивности больше нет
                    //
                    // При входе в цикл создаётся снапшот каждой битовой маски, через него
                    // будет происходить чтение. Оригинал деактивируется (сбрасывается) до
                    // нуля, он будет использоваться для записи
                    //
                    // Каждый спарк привязан через compile-time айди к биту в своей битовой
                    // маске, для каждого реактивного блока (блока который зависит от спарка)
                    // генерируется проверка бита в снапшоте маски. Реактивный блок
                    // запускается если
                    //  - У одного из спарков в блоке есть активный бит в его маске
                    //  - Контекст флэша Build или Navigate
                    // Реактивный блок выполняется всегда при навигации даже если его спарки
                    // не обновлялись
                    //
                    // Дальше идёт код пользователя с генерацией под метки и в конце идёт
                    // проверка всех масок и _fwc_guard

                    let mask_count = self.mask_count.get(&id).unwrap_or(&0);

                    let mut bitmask_strings: Vec<String> = Vec::new();
                    let mut bitmask_clone_strings: Vec<String> = Vec::new();
                    let mut bitmask_check_string = String::new();
                   
                    // Генерация начала цикла реактивности, он нужен чтобы правильно очистить
                    // битовые маски для каждого шага цикла, но при этом иметь возможность
                    // сравнивать бит для проверки изменения реактивности. Это нужно выполнить
                    // для каждой битовой маски
                    for mask_index in 0u8..*mask_count {
                        // Первое, делается клон (копия, так как маска u64) каждой битовой
                        // маски. Он нужен чтобы ЧИТАТЬ его и проверять измненение состояния.
                        // Это локальная переменная которая будет дропнута
                        bitmask_strings.push(format!("let mut _fwc_bitmask{} = 0u64;\n",
                            mask_index + 1));

                        // Первое, делается клон (копия, так как маска u64) каждой битовой
                        // маски. Он нужен чтобы ЧИТАТЬ его и проверять измненение состояния.
                        // Это локальная переменная которая будет дропнута
                        bitmask_clone_strings.push(
                            format!("let _fwc_bitmask{}_clone = _fwc_bitmask{};",
                                mask_index + 1, mask_index + 1));

                        // Второе, обнуляется оригинальная маска после того как сделан клон.
                        // Это нужно чтобы ПИСАТЬ в неё по мере нахождения UpdateSpark. На
                        // следующем шаге цикла (если цикл не будет завершёе из-за превышения
                        // максимального количества итераций или отсуствия обновлений в маске)
                        // эта оригинальная маска где при UpdateSpark будут записаны
                        // обновления будет скопирована для чтения, а сама маска обнулится 
                        // чтобы в неё сами писали. Это самый элегантный способ реализации,
                        // так как, например, точечный сброс бита имеют огромные недостатки
                        bitmask_clone_strings.push(
                            format!("_fwc_bitmask{} = 0;\n", mask_index + 1));

                        // Генерация условия проверки всех битовых масок. Только если все
                        // битовые маски пустые (== 0, не содержат активных битов) то
                        // тогда нужно завершить цикл реактивности, либо если _fwc_guard
                        // больше 64 (Для того чтобы безопасно разрешить циклическую
                        // зависимость)
                        bitmask_check_string.push_str(format!("_fwc_bitmask{} == 0 && ",
                            mask_index + 1).as_str());
                    }

                    // Небольшой хак. Вторая часть выражения сгенерирует код типа
                    // if _fwc_bitmask0 == 0 && {} или if _fwc_bitmask0 == 0 &&
                    // _fwc_bitmask1 == 0 && {}. Чтобы не усложнять кодогенерацию
                    // здесь добавляется true, таким образом результат кодогенерации
                    // выглядит как: if _fwc_bitmask0 == 0 && true { break; }
                    bitmask_check_string.push_str(" true ");

                    let bitmask_statements = bitmask_strings.to_token_streams().unwrap();
                    let bitmask_clone_statements = bitmask_clone_strings.to_token_streams().unwrap();
                    let bitmask_check_expr = parse_str::<Expr>(&bitmask_check_string).unwrap();

                    let generated_block: Block = parse_str(&output).unwrap();
                   
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

                        #generated_block

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
