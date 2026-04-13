// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use syn::visit_mut::VisitMut;
use syn::spanned::Spanned;
use syn::*;
use std::mem::take;
use quote::format_ident;

use super::generator::static_gen::*;

use crate::compiler::codegen::actions::FireworkIR;

pub struct CodegenVisitor<'a> {
    // IR от анализатора, содержит плоские семантические метки для каждого стейтемента,
    // а также содержит снапшот (Мапинг спан -> метка стейтемента)
    pub ir: &'a mut FireworkIR,

    // При каждом входе в функцию проверяется есть ли запись о ней в ir, если есть то
    // этот флаг поднимается, если нет то отпускается. Если он поднят то нужно генерировать
    // код для UI, если нет то это обычная функция. Содержит внутри айди экрана который
    // используется в структуре и экземпляре которые создаются
    pub ui_id: Option<u128>,
}

impl<'a> CodegenVisitor<'a> {
    pub fn new(ir: &'a mut FireworkIR) -> Self {
        Self {
            ir,
            ui_id: None,
        }
    }
}

impl<'a> VisitMut for CodegenVisitor<'a> {
    fn visit_file_mut(&mut self, i: &mut File) {
        let mut new_items = Vec::new();
        
        for item in &mut i.items {
            if let Item::Fn(item_fn) = item {
                let function_name = item_fn.sig.ident.to_string();
                
                // Поиск имени функции в IR, если оно не найдено от find вернёт None в self.ui_id
                // и код для UI не сгенерируется
                self.ui_id = self.ir.screens
                    .iter()
                    .find(|(name, _, _)| name == &function_name)
                    .map(|(_, _, id)| *id);

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
                    let instance = static_declaration(&instance_name, &struct_name_raw, &fields_data[..]);

                    // Парсинг декларации инстанса в syn тип. Это нужно так как quote 
                    // паникует при попытке вставить сырые строки
                    let instance_item: Item = parse_str(&instance).unwrap();

                    let struct_def: Item = parse_quote_spanned!(span=> 
                        struct #struct_name {
                            #(#fields),*
                        }
                    );
                    
                    new_items.push(struct_def);
                    new_items.push(instance_item);
                }
            }
            
            new_items.push(item.clone());
        }
        
        i.items = new_items;
        syn::visit_mut::visit_file_mut(self, i);
    }

    fn visit_stmt_mut(&mut self, i: &mut Stmt) {
        syn::visit_mut::visit_stmt_mut(self, i);
    }
}
