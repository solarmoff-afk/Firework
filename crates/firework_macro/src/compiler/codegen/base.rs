// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::CodeGen;

impl CodeGen {
    /// Добавляет в выходной код декларацию всех элементов верхнего уровня которые
    /// собрал analyze и которые не относятся к ui, например
    ///
    /// pub struct Name {
    ///  // ...
    /// }
    pub(crate) fn inline_items(&self, output: &mut String) {
        for item in self.ir.items.iter() {
            output.push_str(item);
            output.push('\n');
        }

        output.push('\n');
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
    pub(crate) fn inline_block_struct(&self, output: &mut String) {
        for (block_struct, fields) in &self.ir.screen_structs {
            if fields.len() > 0 {
                output.push_str(format!("struct {} {{\n", block_struct).as_str());
                
                for (field_name, field_type) in fields {
                    output.push_str(format!("\t{}: Option<{}>,\n", field_name, field_type).as_str());
                }
              
                // Специальное поле чтобы хранить индекс указателя на функцию экрана в фреймворке 
                output.push_str("\t_fwc_screen_id: Option<usize>,\n");
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
                    "static {}_INSTANCE: firework::OnceCell<{}> = firework::OnceCell::new();\n\n",
                    instance_name, block_struct,
                ).as_str());
            }
        }
    }
}
