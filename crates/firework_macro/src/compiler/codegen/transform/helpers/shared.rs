// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::super::*;

use crate::CompileType;
use crate::compiler::codegen::generator::static_gen;

impl CodegenVisitor<'_> {
    /// Генерирует функцию build, она нужна чтобы инициализировать состояние шейреда.
    /// Вызывается в любой функции чтобы проверить инициализацию спарка и если он None то
    /// инициализировать его
    pub(crate) fn generate_shared_build(&self, id: u128) -> (Vec<TokenStream>, TokenStream) {
        let struct_name = format!("ApplicationUiBlockStruct{}", id);
        let mut statements: Vec<TokenStream> = Vec::new();

        for field in &self.ir.shared.state {
            let field_name = format!("spark_{}", field.id);
            let set_field_str = static_gen::set_field(&struct_name, &field_name, &field.init);
            statements.push(CodeBuilder::convert_string_to_syn(&set_field_str));
        }

        let default = Vec::new();
        let fields_data = self.ir.screen_structs
            .get(&struct_name)
            .unwrap_or(&default);

        let build_check = static_gen::init_instance(
            &struct_name.to_uppercase(), &struct_name, &fields_data);
        
        let build_check_statement = CodeBuilder::convert_string_to_syn(&build_check);

        (statements, build_check_statement)
    }

    /// Обрабатывает сахарные метки для разделямого состояния, например:
    ///
    /// ```ignore
    /// state! {
    ///     #[read] #[write]
    ///     my_state: i32 = 10,
    /// }
    /// ```
    pub(crate) fn resolve_shared_desugar_attr(&self, new_items: &mut Vec<Item>) {
        if let Some(id) = self.ui_id && matches!(self.flags.compile_type, CompileType::Shared) {
            for state in &self.ir.shared.state {
                for attr in &state.attributes {
                    let field_name = &state.name;
                    let field_type: Type = syn::parse_str(&state.spark_type).unwrap();
                    let field_id = state.id;
                   
                    // Генерирует геттер для этого состояния, имя функции устанавливается
                    // как get_{}, где {} это имя состояния. Геттер возвращает &'static T 
                    // на это состояние
                    if attr == "read" {
                        let getter = self.desugar_shared_read(
                            state.span,
                            field_id as u128,
                            field_name,
                            &field_type,
                            id,
                        );
                        new_items.push(getter);
                    }
                    
                    // Генерирует сеттер для этого состояния который активирует эффекты
                    // которые подписаны на него (функциональные). Принимает значение типа
                    // T и ничего не возвращает, имя set_{}
                    if attr == "write" {
                        let setter = self.desugar_shared_write(
                            state.span,
                            field_id as u128,
                            field_name,
                            &field_type,
                            id,
                        );
                        new_items.push(setter);
                    }
                }
            }
        }
    }
}
