// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use proc_macro2::TokenStream;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::*;

/// Обрабатывает виртуальные макросы и выполняет их развёртку в набор стейтементов syn
/// так как виртуальных макросов (маркеров) в реальности нет
pub struct MacroResolver;

impl MacroResolver {
    /// Анализирует стейтемент и по имени виртуального макроса определяет нужно ли выполнить
    /// развёртку кода и какую именно
    pub fn expand(statement: &Stmt) -> Option<Vec<Stmt>> {
        let statement_macro = match statement {
            // Нужны только макросы как стейтементы, макросы выражения и не макросы вообще
            // MacroResolver не должен обрабатывать
            Stmt::Macro(m) => m,

            _ => return None,
        };

        // Имя виртуального макроса (маркера)
        let identifier = statement_macro.mac.path.get_ident()?;

        // Какой это именно маркер и какую функцию развёртки нужно применить
        match identifier.to_string().as_str() {
            // Маркер эффект
            "effect" => Self::expand_effect_macro(&statement_macro.mac.tokens),

            // Это не маркер или его не нужно развёртывать на этом этапе
            _ => None,
        }
    }

    /// Развёртка маркера effect!(spark, {}), маркер анализируется и в код попадает только
    /// блок внутри (последний аргумент). Анализатор не пропустит маркер effect где блок
    /// это не последний аргумент, поэтому всё нормально
    fn expand_effect_macro(tokens: &TokenStream) -> Option<Vec<Stmt>> {
        // Парсинг по запятой среди токенов вызова маркера
        let parser = Punctuated::<Expr, Token![,]>::parse_terminated;
        let punctuated = parser.parse2(tokens.clone()).ok()?;
        let arguments: Vec<Expr> = punctuated.into_iter().collect();

        // Последний аргумент всегда блок, это гарантирует анализатор (первый проход)
        // иначе была бы ошибка и компилятор не запустил бы кодогенерацию
        let last_argument = arguments.last()?;

        let block_expression = match last_argument {
            Expr::Block(block_expression) => block_expression,
            _ => return None,
        };

        // Последний блок
        Some(block_expression.block.stmts.clone())
    }
}
