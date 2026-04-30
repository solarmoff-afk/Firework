// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

mod nodes;

use proc_macro2::{TokenStream, Span};
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;

use super::ir::{FireworkIR, FireworkAction, FireworkStatement};
use super::generator::static_gen;
use super::generator::bitmask_gen::*;
use super::transform::traits::{ToStmt, ToExpr};
use super::consts::CHECK_NAVIGATE;

pub struct CodeBuilder {
    /// Токены которые вставляются под конец функции за пределами цикла реактивности
    pub tokens: Vec<TokenStream>,
    
    ir: FireworkIR,
}

impl CodeBuilder {
    pub fn new(ir: FireworkIR) -> Self {
        Self {
            tokens: Vec::new(),
            ir,
        }
    }

    pub fn convert_string_to_syn(code: &str) -> TokenStream { 
        code.parse().expect(format!("Invalid syntax: {}", code).as_str())
    }

    /// Сборка токенов из реального стейтемента и набора семантических меток. Через quote 
    /// генерируется набор токенов и возвращается после чего вставляется на место стейтемента.
    /// Ноды сами решают использовать ли оригинальный стейтемент
    pub fn build(
        &mut self, stmt: &syn::Stmt, statements: &[FireworkStatement], 
        mut processed_body: TokenStream,
    ) -> TokenStream {
        // Спан нужен для того чтобы вставить код в нужное место для правильных ошибок
        // rustc
        let span = stmt.span();
        
        let mut final_tokens = TokenStream::new();
        let mut is_body_handled = false;

        // Ноды которые полностью меняют оригинальный код перезаписывая его
        for statement in statements {
            let struct_name = format!("ApplicationUiBlockStruct{}", statement.screen_index);
            let mut temp_tokens = TokenStream::new();
            
            let tokens = &mut temp_tokens;
            if self.node_initial_spark(span, struct_name.clone(), tokens, &statement) ||
               self.node_spark_ref(span, struct_name.clone(), tokens, &statement) ||
               self.node_widget_block(span, struct_name.clone(), tokens, &statement) 
            {
                processed_body = temp_tokens; 
                is_body_handled = true;
                break; 
            }
        }

        for statement in statements {
            let mut temp_tokens = TokenStream::new();
            if let FireworkAction::UpdateSpark(..) = statement.action {
                if self.node_update_spark(span, &mut temp_tokens, &statement, &processed_body) {
                    processed_body = temp_tokens;
                    is_body_handled = true;
                }
            }
        }

        for statement in statements {
            let mut temp_tokens = TokenStream::new();
            if let FireworkAction::ReactiveBlock(..) = statement.action {
                if self.node_reactive_block(span, &mut temp_tokens, &statement, &processed_body) {
                    processed_body = temp_tokens;
                    is_body_handled = true;
                }
            }
        }

        for statement in statements {
            let mut temp_tokens = TokenStream::new();
            if let FireworkAction::DynamicLoopBegin(..) = statement.action {
                let struct_name = format!("ApplicationUiBlockStruct{}", statement.screen_index);
                if self.node_dynamic_list(span, &mut temp_tokens, struct_name, &statement, &processed_body) {
                    processed_body = temp_tokens;
                    is_body_handled = true;
                }
            }
        }

        // Возврат владения в статику
        let mut drop_tokens = TokenStream::new();
        for statement in statements {
            if let FireworkAction::DropSpark { .. } = statement.action {
                let struct_name = format!("ApplicationUiBlockStruct{}",
                    statement.screen_index);

                self.node_drop_spark(span, struct_name, &mut drop_tokens, &statement);
            }
        }

        // Финальная сборка
        if is_body_handled && !processed_body.is_empty() {
            final_tokens.extend(processed_body);
        } else {
            final_tokens.extend(quote_spanned!(span=> {
                #processed_body
            }));
        }

        // DropSpark всегда идёт вне контекстных условий
        final_tokens.extend(drop_tokens);
        final_tokens
    }

    /// Выполняется при выходе из функции чтобы подготовить билдер к генерации кода для
    /// следующей функции
    pub fn function_end(&mut self) {
        self.tokens.clear();
    }

    pub(crate) fn generate_check_spark_bit(&self, code: &mut String, id: usize) {
        self.generate_check(code, id, "_fwc_bitmask", "_clone");
    }

    pub(crate) fn generate_check_widget_bit(&self, code: &mut String, id: usize) {
        self.generate_check(code, id, "_fwc_widget_bitmask", "");
    }

    fn generate_check(&self, code: &mut String, id: usize, mask_name: &str, mask_suffix: &str) {
        // Получение маски на основе айди спарка
        let mask = get_spark_mask(id);
        let id_in_mask = normalize_bit_index(id);

        code.push_str(check_flag(
            // Имя маски
            format!("{}{}{}", mask_name, mask, mask_suffix).as_str(),
            
            // Индекс внутри этой маски
            id_in_mask,
        ).as_str());
    }

    /// Метод для генерации деактивации битов в маске при изменении спарка который
    /// был в условии от которых зависит декларация условного виджета
    pub(crate) fn generate_widget_spark_update(
        &self,
        statement: &FireworkStatement,
        spark_id: &usize,
    ) -> TokenStream {
        let mut update_widgets_statements = TokenStream::new();

        if let Some(map) = self.ir.screen_maybe_widgets.get(&statement.screen_index) { 
            if let Some(widgets) = map.spark_widget_map.get(spark_id) {
                for widget in widgets {
                    // Битовая маска этого условного виджета
                    let mask = get_spark_mask(*widget);

                    let mask_statement = format!("{};", unset_flag(
                        format!("_fwc_widget_bitmask{}", mask).as_str(), 
                        normalize_bit_index(*widget),
                    )).to_stmt().unwrap(); 

                    update_widgets_statements.extend(quote! {
                        #mask_statement
                    });
                }
            }
        }

        update_widgets_statements
    }
}
