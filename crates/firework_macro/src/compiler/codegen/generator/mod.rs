// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

mod base;
mod static_gen;

use std::collections::HashMap;
use proc_macro2::TokenTree;
use rand::Rng;

use super::actions::{FireworkIR, FireworkStatement, FireworkAction};
use super::consts::SCREEN_HEADER;

// NOTE: Дополнительные методы реализованы в base.rs
pub struct CodeGen {
    pub ir: FireworkIR,

    // Хэш мап для хранения результатов кодогенерации для каждого экрана
    screen_map: HashMap<String, (String, u64)>,
}

impl CodeGen {
    pub fn new(ir: FireworkIR) -> Self {
        Self {
            ir,
            screen_map: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        let mut output = String::from("");

        self.inline_items(&mut output);
        self.inline_block_struct(&mut output);

        self.make_screens_body(1);
        self.inline_screens(&mut output);

        for statement in self.ir.statements.iter() {
            // println!("{:#?}", statement);
        }

        println!("Output:\n{}", output);
    }
 
    fn inline_screens(&mut self, output: &mut String) {
        for (screen_name, screen_signature, screen_id) in self.ir.screens.iter() { 
            output.push_str(format!("{} {{\n", screen_signature).as_str());
            
            let struct_name = format!("ApplicationUiBlockStruct{}", screen_id);
            let instance_name = struct_name.to_uppercase();
           
            // Проверка является ли это первым вызовом функции, так как на каждый экран
            // (функцию) идёт свой экземпляр то можно проверять по нему 
            output.push_str(static_gen::is_first_call(&instance_name).as_str());
           
            // Инициализация если экземпляр ещё не инициализирован
            output.push_str(static_gen::init_instance(&instance_name, screen_name).as_str());

            // Определение 
            // output.push_str();

            // Добавляем код экрана
            if let Some(screen_code) = self.screen_map.get(screen_name) {
                output.push_str(&screen_code.0);
            }
            
            output.push_str("}\n\n");
        }
    }

    fn make_screens_body(&mut self, depth: usize) {
        for statement in self.ir.statements.iter() {
            let depth = "\t".repeat(depth + statement.scope.depth);
            if !self.screen_map.contains_key(&statement.screen_name) {
                let id: u64 = rand::thread_rng().gen_range(0..=u64::MAX); 

                self.screen_map.insert(statement.screen_name.clone(), (String::from(SCREEN_HEADER), id));
            }

            if let Some(screen_code) = self.screen_map.get_mut(&statement.screen_name) {
                // Виджеты не нужно добавлять в вывод
                if matches!(statement.action, FireworkAction::WidgetBlock(..)) {
                    screen_code.0.push_str(format!("{}// Widget\n", depth).as_str());
                    continue;
                }

                screen_code.0.push_str(format!("{}{}\n", depth, statement.string).as_str());
            }
        }
    }
}
