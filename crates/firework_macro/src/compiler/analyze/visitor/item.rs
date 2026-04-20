// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use syn::punctuated::Punctuated;
use quote::quote;

pub use super::super::*;
use super::super::spark::GlobalState;

use crate::CompileType;

impl<'ast> Analyzer {
    /// Анализ макросов на верхнем уровне
    pub(crate) fn analyze_item_macro(&mut self, i: &'ast ItemMacro) {
        // Если комплиятор сейчас компилирует shared и это макрос state, то это
        // объявление глобального состояния для shared блока
        if matches!(self.context.flags.compile_type, CompileType::Shared) {
            if i.mac.path.is_ident("state") {
                let parser = Punctuated::<GlobalState, Token![,]>::parse_terminated;
                
                if let Ok(fields) = i.mac.parse_body_with(parser) {
                    for field in &fields {
                        // Так как quote не умеет вставлять #name.field (ошибка компиляции)
                        // нужно клонировать значения из полей в обычные переменные и
                        // вставлять их, так как #name quote обрабатывает отлично 
                        let raw_type = field.spark_type.clone();
                        let raw_init = field.init.clone();

                        let name = field.name.to_string();
                        let spark_type = quote!(#raw_type).to_string(); 
                        let init = quote!(#raw_init).to_string();
                        
                        println!("Name: {}, type: {}, init: {}", name, spark_type, init);
                    }
                }
            }
        }

        visit::visit_item_macro(self, i);
    }
}

