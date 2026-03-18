// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use std::collections::HashMap;
use proc_macro2::TokenTree;
use rand::Rng;

use super::actions::{FireworkIR, FireworkStatement, FireworkAction};
use super::consts::SCREEN_HEADER;

pub struct CodeGen {
    pub ir: FireworkIR,

    // Хэш мап для хранения результатов кодогенерации для каждого экрана
    screen_map: HashMap<String, (String, u16)>,
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

    /// Эта функция берёт информацию из IR и создаёт верхушку результата кодогенерации
    /// где структура ui блока, пример
    ///
    /// struct ApplicationUiBlockStruct1 {
	  ///     spark_0: Option<Vec < u32 >>,
    ///	    widget_object_3: Option<firework::RectSkin>,
    /// }
    ///
    /// Используется Option так как на этапе создания статического экземпляра значения
    /// полей могут быть зависимы от внешних данных, поэтому используется None. Пример
    /// статического экземпляра:
    ///
    /// static mut APPLICATIONUIBLOCKSTRUCT1_INSTANCE: ApplicationUiBlockStruct1 = ApplicationUiBlockStruct1 {
    ///     spark_0: None,
	  ///     widget_object_3: None,
    /// }
    fn inline_block_struct(&self, output: &mut String) {
        for (block_struct, fields) in &self.ir.screen_structs {
            if fields.len() > 0 {
                output.push_str(format!("struct {} {{\n", block_struct).as_str());
                
                for (field_name, field_type) in fields {
                    output.push_str(format!("\t{}: Option<{}>,\n", field_name, field_type).as_str());
                }
                
                output.push_str("}\n\n"); 
            } else {
                output.push_str(format!("struct {};\n\n", block_struct).as_str());
            }
        }

        // Статический экземпляр (для доступа нужен unsafe, это нормально так как ui всегда
        // однопоточный), а RefCell добавляет оверхед и раздувает результат кодогенерации
      
        for (block_struct, fields) in &self.ir.screen_structs {
            let instance_name = block_struct.to_uppercase();

            if fields.len() > 0 {
                output.push_str(format!(
                    "static mut {}_INSTANCE: {} = {} {{\n",
                    instance_name, block_struct, block_struct,
                ).as_str());
                
                for (field_name, _) in fields {
                    output.push_str(format!("\t{}: None,\n", field_name).as_str());
                }
                
                output.push_str("}\n\n");
            } else {
                output.push_str(format!(
                    "static mut {}_INSTANCE: {} = {};\n",
                    instance_name, block_struct, block_struct,
                ).as_str());
            }
        }
    }

    fn inline_items(&self, output: &mut String) {
        for item in self.ir.items.iter() {
            output.push_str(item);
            output.push('\n');
        }

        output.push('\n');
    }

    fn inline_screens(&self, output: &mut String) {
        for (screen_name, screen_signature, screen_index) in self.ir.screens.iter() {
            output.push_str(format!(
                "fn _fwc_{}_caller() {{\n\t{}();\n}}\n\n", screen_name, screen_name).as_str()
            );

            output.push_str(format!("{} {{\n", screen_signature).as_str());

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
                let id: u16 = rand::thread_rng().gen_range(0..=u16::MAX);
                
                let to_inline = format!(
                    "{}let _FWC_SCREEN_ID: u32 = {};\n{}",
                    depth, id, SCREEN_HEADER
                );

                self.screen_map.insert(statement.screen_name.clone(), (String::from(to_inline), id));
            }

            if let Some(screen_code) = self.screen_map.get_mut(&statement.screen_name) {
                screen_code.0.push_str(format!("{}{}\n", depth, statement.string).as_str());
            }
        }
    }
}
