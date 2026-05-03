// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

#[cfg(feature = "trace")]
use tracing::instrument;

use super::super::traits::ToTokenStreams;
use super::super::*;

use crate::compiler::codegen::consts::CHECK_EVENT_INCODE;
use crate::compiler::codegen::generator::static_gen;

impl CodegenVisitor<'_> {
    #[cfg_attr(feature = "trace", instrument(skip_all, fields(id = ?id)))]
    pub fn generate_widgets_mask(&self, id: u128, struct_name: &str) -> Vec<TokenStream> {
        let mask_count = self.widget_mask_count.get(&id).unwrap_or(&0);

        let mut bitmask_strings: Vec<String> = Vec::new();

        for mask_index in 0u8..*mask_count {
            // Первая битовая маска позволяет проверить есть ли изменение
            let mask_name = format!("_fwc_widget_bitmask{}", mask_index + 1);
            bitmask_strings.push(format!("let mut {} = 0u64;\n", mask_name));

            let copy_field_str = static_gen::copy_field(struct_name, &mask_name, &mask_name);

            bitmask_strings.push(format!(
                "if {} {{ {} }}",
                CHECK_EVENT_INCODE, copy_field_str
            ));
        }

        bitmask_strings.to_token_streams().unwrap()
    }
}
