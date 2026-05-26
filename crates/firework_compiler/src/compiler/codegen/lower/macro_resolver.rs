// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
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
            "effect" => {
                Self::expand_effect_macro(&statement_macro.mac.tokens, statement_macro.span())
            }

            // Удаление дескриптора лайаута так как он не нужен в выходном коде
            "layout" => {
                Some(Vec::new())
            }

            // Это не маркер или его не нужно развёртывать на этом этапе
            _ => None,
        }
    }

    /// Развёртка маркера effect!(spark, {}), маркер анализируется и в код попадает только
    /// блок внутри (последний аргумент). Анализатор не пропустит маркер effect где блок
    /// это не последний аргумент, поэтому всё нормально
    fn expand_effect_macro(tokens: &TokenStream, macro_span: Span) -> Option<Vec<Stmt>> {
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

        // HACK: Эффект заменяется на всегда верное условие со спаном оригинального стейтемента
        // чтобы трансформ нашёл метки в IR для него и отправил в code_builder
        let stmts = &block_expression.block.stmts;
        let expanded = quote_spanned!(macro_span=>
            if true {
                #(#stmts)*
            }
        );
        let stmt: Stmt = syn::parse2(expanded).ok()?;

        // Последний блок
        Some(vec![stmt])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use quote::{quote, ToTokens};

    fn parse_stmt(tokens: TokenStream) -> Stmt {
        syn::parse2(tokens).unwrap()
    }

    #[test]
    fn test_expand_effect_macro_basic() {
        let input = quote! {
            effect!(spark, {
                let x = 42;
                println!("Hello");
            });
        };
        let stmt = parse_stmt(input);
        
        let result = MacroResolver::expand(&stmt);
        assert!(result.is_some());
        
        let expanded_stmts = result.unwrap();
        assert_eq!(expanded_stmts.len(), 1);
        
        let expanded_str = expanded_stmts[0].to_token_stream().to_string();
        assert!(expanded_str.contains("if true"));
        assert!(expanded_str.contains("let x = 42"));
        assert!(expanded_str.contains("println ! (\"Hello\")"));
    }

    #[test]
    fn test_expand_effect_macro_with_multiple_effects() {
        let input = quote! {
            effect!(spark, fallback, {
                let y = 10;
            });
        };
        let stmt = parse_stmt(input);
        
        let result = MacroResolver::expand(&stmt);
        assert!(result.is_some());
        
        let expanded_stmts = result.unwrap();
        assert_eq!(expanded_stmts.len(), 1);
        assert!(expanded_stmts[0].to_token_stream().to_string().contains("let y = 10"));
    }

    #[test]
    fn test_expand_effect_macro_with_complex_block() {
        let input = quote! {
            effect!(test, {
                let a = 1;
                let b = 2;
                let c = a + b;
                if c > 0 {
                    println!("Positive");
                }
            });
        };
        let stmt = parse_stmt(input);
        
        let result = MacroResolver::expand(&stmt);
        assert!(result.is_some());
        
        let expanded = result.unwrap();
        let expanded_code = expanded[0].to_token_stream().to_string();
        assert!(expanded_code.contains("if true"));
        assert!(expanded_code.contains("let a = 1"));
        assert!(expanded_code.contains("let b = 2"));
        assert!(expanded_code.contains("let c = a + b"));
        assert!(expanded_code.contains("println ! (\"Positive\")"));
    }

    #[test]
    fn test_expand_layout_macro_removes() {
        let input = quote! {
            layout!(Some, Args, Here);
        };
        let stmt = parse_stmt(input);
        
        let result = MacroResolver::expand(&stmt);
        assert!(result.is_some());
        
        let expanded_stmts = result.unwrap();
        assert!(expanded_stmts.is_empty());
    }

    #[test]
    fn test_ignores_non_macro_statements() {
        let input = quote! {
            let x = 5;
        };
        let stmt = parse_stmt(input);
        
        let result = MacroResolver::expand(&stmt);
        assert!(result.is_none());
    }

    #[test]
    fn test_ignores_unknown_macros() {
        let input = quote! {
            unknown_macro!(something);
        };
        let stmt = parse_stmt(input);
        
        let result = MacroResolver::expand(&stmt);
        assert!(result.is_none());
    }

    #[test]
    fn test_effect_macro_without_block_returns_none() {
        let input = quote! {
            effect!(spark, 42);
        };
        let stmt = parse_stmt(input);
        
        let result = MacroResolver::expand(&stmt);
        assert!(result.is_none());
    }

    #[test]
    fn test_effect_macro_with_empty_block() {
        let input = quote! {
            effect!(spark, {});
        };
        let stmt = parse_stmt(input);
        
        let result = MacroResolver::expand(&stmt);
        assert!(result.is_some());
        
        let expanded_stmts = result.unwrap();
        assert_eq!(expanded_stmts.len(), 1);
        
        let expanded_code = expanded_stmts[0].to_token_stream().to_string();
        assert!(expanded_code.contains("if true"));
    }

    #[test]
    fn test_effect_macro_with_multiple_arguments_before_block() {
        let input = quote! {
            effect!(spark, some_arg, another_arg, {
                let x = 1;
            });
        };
        let stmt = parse_stmt(input);
        
        let result = MacroResolver::expand(&stmt);
        assert!(result.is_some());
        
        let expanded = result.unwrap();
        assert_eq!(expanded.len(), 1);
        assert!(expanded[0].to_token_stream().to_string().contains("let x = 1"));
    }

    #[test]
    fn test_invalid_effect_macro_syntax() {
        let input1 = quote! {
            effect!(spark {});
        };
        let parse_result1 = syn::parse2::<Stmt>(input1);
        
        if let Ok(stmt) = parse_result1 {
            let result = MacroResolver::expand(&stmt);
            assert!(result.is_none(), "Expected None for invalid macro syntax");
        } else {
            assert!(parse_result1.is_err());
        }
        
        let input2 = quote! {
            effect!();
        };
        let parse_result2 = syn::parse2::<Stmt>(input2);
        if let Ok(stmt) = parse_result2 {
            let result = MacroResolver::expand(&stmt);
            assert!(result.is_none(), "Expected None for effect without arguments");
        }
        
        let input3 = quote! {
            effect!(spark, 42, "not a block");
        };
        let parse_result3 = syn::parse2::<Stmt>(input3);
        if let Ok(stmt) = parse_result3 {
            let result = MacroResolver::expand(&stmt);
            assert!(result.is_none(), "Expected None when last argument is not a block");
        }
    }

    #[test]
    fn test_handles_multiple_statements_in_effect() {
        let input = quote! {
            effect!(test, {
                let x = 1;
                let y = 2;
                let z = x + y;
                println!("Result: {}", z);
            });
        };
        let stmt = parse_stmt(input);
        
        let result = MacroResolver::expand(&stmt);
        assert!(result.is_some());
        
        let expanded = result.unwrap();
        let expanded_code = expanded[0].to_token_stream().to_string();
        
        assert!(expanded_code.contains("let x = 1"));
        assert!(expanded_code.contains("let y = 2"));
        assert!(expanded_code.contains("let z = x + y"));
        assert!(expanded_code.contains("println ! (\"Result: {}\" , z)"));
    }

    #[test]
    fn test_effect_macro_with_return_statement() {
        let input = quote! {
            effect!(test, {
                let x = 42;
                return x;
            });
        };
        let stmt = parse_stmt(input);
        
        let result = MacroResolver::expand(&stmt);
        assert!(result.is_some());
        
        let expanded = result.unwrap();
        let expanded_code = expanded[0].to_token_stream().to_string();
        assert!(expanded_code.contains("if true"));
        assert!(expanded_code.contains("let x = 42"));
        assert!(expanded_code.contains("return x"));
    }
}
