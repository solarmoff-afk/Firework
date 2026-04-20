// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use proc_macro2::Span;
use quote::quote;
use syn::parse_quote;

pub use super::super::*;

use crate::CompileType;

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
                self.functions_count += 1;
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

                    let struct_name_raw = format!("ApplicationUiBlockStruct{}", id);
                    let struct_name = format_ident!("ApplicationUiBlockStruct{}", id);
                    
                    // Только для shared
                    let build_name = format_ident!("_fwc_fn_build{}", id);
                    
                    let mut fields: Vec<Field> = Vec::new();
                    let fields_data = self.generate_fields(id, &mut fields, span);

                    // Генерация статического экземпляра. Если используется safety-multitrhead
                    // фича то static_gen генерирует OnceLock + Mutex для безопасной работы
                    // из нескольких поток, если safety-multitrhead нет то используется
                    // static mut и unsafe
                    let instance_name = struct_name_raw.to_uppercase();
                    let instance = static_declaration(&instance_name, &struct_name_raw, &fields_data);
                    let instance_item: Item = parse_str(&instance).unwrap();

                    // Структура экрана где хранится состояние, компоненты и виджеты
                    let struct_def: Item = parse_quote_spanned!(span=> 
                        struct #struct_name {
                            #(#fields),*
                        }
                    );
                  
                    // Проверка можно ли генерировать структуру сейчас, в Shared режиме
                    // компиляции нужна только одна структура так как состояние глобальное
                    // поэтому после первой генерации в Shared режиме генерировать структуру
                    // и экземпляр больше нельзя
                    if self.should_generate_struct() {
                        new_items.push(struct_def);
                        new_items.push(instance_item);
                        
                        if matches!(self.flags.compile_type, CompileType::Shared) {
                            let (build_statements, build_check) = self.generate_shared_build(id);

                            let tokens = quote! {
                                fn #build_name () {
                                    let mut _fwc_build = false;

                                    #build_check

                                    if _fwc_build {
                                        #(#build_statements)*
                                    }
                                }
                            };
                            
                            // SAFETY: Код явлется абсолютно валидным, build_statements в
                            // случае синтаксической ошибки были бы отбракованы на этапе
                            // generate_shared_build через функцию из CodeBuilder из-за чего
                            // unwrap здесь безопасен
                            let item: Item = syn::parse2(tokens).unwrap();
                            new_items.push(item);
                        }
                    }
                    
                    // Оригинальное тело функции (уже трансформированное), так как block
                    // не реализует Default нужно использовать std::mem::replace, идёт
                    // парсинг обычного пустого блока чтобы заменить на него оригинал, а
                    // оригинальые данные забрать сюда чтобы избежать клонирования
                    let original_block = std::mem::replace(&mut item_fn.block, parse_quote!({})); 

                    let reactive_output = self.generate_reactive(id);
                    let generated_block = self.generate_flash_pass(id, &function_name);

                    let bitmask_statements = reactive_output.bitmask_statements;
                    let bitmask_clone_statements = reactive_output.bitmask_clone_statements;
                    let bitmask_check_expr = reactive_output.bitmask_check_expr; 

                    let is_shared = matches!(self.flags.compile_type, CompileType::Shared);
                    let init_code = if !is_shared {
                        quote! {
                            let mut _fwc_build = false;
                            #generated_block
                        }
                    } else {
                        // Если это shared то для каждой функции нужно сначала (в первой фазе)
                        // вызвать build функцию чтобы проверить инициализацию и если спарки
                        // ещё не инициализированы на уровне state! {} то нужно их
                        // инициализировать
                        quote! {
                            #build_name();
                        }
                    };

                    // Если цикл совершил более 64 итераций (хардкод )то происходит выход
                    // из него это делается после добавления единицы к итерациям чтобы не
                    // отнимать единицу
                    // (64 - 1 = 63) от максимального количества итераций, так как:
                    //  - Нулевой шаг, +1, 1 итерация
                    //  - Первый шаг,  +1, 2 итерация
                    //  - 63 шаг, +1,  +1, 64 итерация, условие сработало 
                    item_fn.block = parse_quote_spanned!(span=> {
                        let mut _fwc_event = firework_ui::LifeCycle::Navigate;
                        #init_code

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

                self.visit_item_mut(&mut other_item);
                
                new_items.push(other_item);
            }
        }
        
        i.items = new_items;
    }

    /// Генерирует набор полей для вставки по ссылке на fields и возвращает вектор сырых
    /// полей (Имя, тип)
    fn generate_fields(&self, id: u128, fields: &mut Vec<Field>, span: Span) -> Vec<(String, String)> {
        // Вектор полей структуры, хранит кортежи (имя, тип). Они собраны
        // анализатором для имени структуры ApplicationUiBlockStruct{id}
        let default = Vec::new();
        let fields_data = self.ir.screen_structs
            .get(&format!("ApplicationUiBlockStruct{}", id))
            .unwrap_or(&default);

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

        fields_data.to_vec()
    }

    /// Проверяет нужно ли геенрировать структуру для этой функции
    fn should_generate_struct(&self) -> bool {
        match self.flags.compile_type {
            // В Shared режиме структура нужна только первой функции
            CompileType::Shared => self.functions_count == 1,

            // В обычном режиме структура нужная каждой функции так как каждая функция
            // это отдельный экран
            _ => true,
        }
    }
}
