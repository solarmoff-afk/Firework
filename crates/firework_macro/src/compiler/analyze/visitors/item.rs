// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use quote::quote;
use syn::punctuated::Punctuated;

use super::super::expr::spark::GlobalState;
pub use super::super::*;

use crate::CompileType;
use crate::compiler::codegen::ir::FireworkSharedState;

impl<'ast> Analyzer {
    /// Анализ макросов на верхнем уровне
    pub(crate) fn analyze_item_macro(&mut self, i: &'ast ItemMacro) {
        // Если комплиятор сейчас компилирует shared и это макрос state, то это
        // объявление глобального состояния для shared блока
        if matches!(self.context.flags.compile_type, CompileType::Shared) {
            if i.mac.path.is_ident("state") {
                // Снапшот текущего имени функции
                let old_function_name = self.function_name.clone();

                // Чтобы добавление полей прошло проверку на наличие какой-то структуры
                // нужно вписать в поле которое отвечает за имя текущей функции заглушку,
                // для state внутри shared используется _fwc_shared_build
                self.function_name = Some("_fwc_shared_build".to_string());

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

                        self.context.ir.shared.state.push(FireworkSharedState {
                            name,
                            spark_type: spark_type.clone(),
                            init,
                            span: field.span,
                            id: self.context.spark_counter,
                            attributes: field.attributes.clone(),
                        });

                        self.add_field_to_struct(
                            format!("spark_{}", self.context.spark_counter),
                            spark_type,
                        );
                        self.context.spark_counter += 1;
                    }
                }

                // Возврат старого имени
                self.function_name = old_function_name;
            }
        }

        visit::visit_item_macro(self, i);
    }
}
