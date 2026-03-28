// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

mod base;
mod static_gen;

use std::collections::HashMap;
use rand::Rng;

use super::actions::{FireworkIR, FireworkAction};
use super::consts::*;

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

        for _statement in self.ir.statements.iter() {
            // println!("{:#?}", _statement);
        }

        println!("Output:\n{}", output);
    }
 
    fn inline_screens(&mut self, output: &mut String) {
        for (screen_name, screen_signature, screen_id) in self.ir.screens.iter() { 
            output.push_str(format!("{} {{\n", screen_signature).as_str());
            
            let struct_name = format!("ApplicationUiBlockStruct{}", screen_id);
            let instance_name = struct_name.to_uppercase();
            
            output.push_str("\t// Phase 1: Init\n\n");
           
            // Проверка является ли это первым вызовом функции, так как на каждый экран
            // (функцию) идёт свой экземпляр то можно проверять по нему 
            output.push_str(static_gen::is_first_call(&instance_name).as_str());
            output.push_str("\tlet mut _fwc_build = false;\n");
           
            // Инициализация если экземпляр ещё не инициализирован
            output.push_str(static_gen::init_instance(&instance_name, screen_name).as_str());

            output.push_str(format!("{}",CHECK_EVENT).as_str());
            
            // Устанавливает фокус на этот экран
            output.push_str(format!("{}", SET_FOCUS).as_str());
            
            output.push_str("\n\t// Phase 2: Navigate/Build code\n");
            
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

            let struct_name = format!("ApplicationUiBlockStruct{}", statement.scope.screen_index);
            if let Some(screen_code) = self.screen_map.get_mut(&statement.screen_name) {
                match &statement.action {
                    FireworkAction::InitialSpark { id, expr_body, name, .. } => {
                        let field_name = format!("spark_{}", id);
                        
                        screen_code.0.push_str(format!(
                            "{}if matches!(_fwc_event, firework::LifeCycle::Build) {{\n", depth,
                        ).as_str());
                        
                        screen_code.0.push_str(&static_gen::set_field(
                            &struct_name,
                            &field_name,
                            &expr_body,
                        ));
                        
                        screen_code.0.push_str(format!("{}}}\n", depth).as_str());
                        
                        // Флаг для того чтобы в 4 фазе найти грязные спарки
                        screen_code.0.push_str(format!("{}let mut _fwc_{}_dirty = false;\n", depth, field_name).as_str());
                        
                        // Снятие владения из структуры
                        let getter = format!("{}_INSTANCE.{}", struct_name, field_name);
                        screen_code.0.push_str(format!("{}let mut {} = unsafe {{ {}.take().unwrap(); }}\n", depth, name, getter).as_str());
                    },

                    _ => {
                        // Делаем инлайн изначальной строки только если у нас нет специальной логики для
                        // этого действия из FireworkAction
                        screen_code.0.push_str(format!("{}{}\n", depth, statement.string).as_str());
                    },
                };
            }
        }
    }
}
