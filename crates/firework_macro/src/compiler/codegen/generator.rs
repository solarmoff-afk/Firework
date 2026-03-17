// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use proc_macro2::TokenTree;

use super::actions::{FireworkIR, FireworkStatement, FireworkAction};

pub struct CodeGen {
    pub ir: FireworkIR,
}

impl CodeGen {
    pub fn new(ir: FireworkIR) -> Self {
        Self {
            ir,
        }
    }

    pub fn run(&self) {
        let mut output = String::from("");

        self.inline_items(&mut output);
        self.inline_block_struct(&mut output);

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
}
