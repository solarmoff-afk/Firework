// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::super::traits::ToTokenStreams;
use super::super::*;

impl CodegenVisitor<'_> {
    pub fn generate_widgets_mask(&self, id: u128) -> Vec<TokenStream> {
        let mask_count = self.widget_mask_count.get(&id).unwrap_or(&0);

        let mut bitmask_strings: Vec<String> = Vec::new();
        
        for mask_index in 0u8..*mask_count {
            // Первая битовая маска позволяет проверить есть ли изменение у 
            bitmask_strings.push(format!("let mut _fwc_widget_bitmask{} = 0u64;\n",
                mask_index + 1));
        }

        let bitmask_statements = bitmask_strings.to_token_streams().unwrap();
        bitmask_statements
    }
}
