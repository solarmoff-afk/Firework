// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::*;

use crate::compiler::codegen::generator::static_gen;

impl CodegenVisitor<'_> {
    pub(crate) fn generate_shared_build(&self, id: u128) -> Vec<TokenStream> {
        let struct_name = format!("ApplicationUiBlockStruct{}", id);
        
        let mut statements: Vec<TokenStream> = Vec::new();

        for field in &self.ir.shared_state {
            let field_name = format!("spark_{}", field.id);
            let set_field_str = static_gen::set_field(&struct_name, &field_name, &field.init);

            statements.push(CodeBuilder::convert_string_to_syn(&set_field_str));
        }

        statements
    }
}
