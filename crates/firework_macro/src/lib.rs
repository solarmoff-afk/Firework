// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

extern crate proc_macro;

mod compiler;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Result};

use compiler::run_firework_compiler;

/// Структура абстрактного синтаксического дерева. Здесь хранятся токены после
/// парсинга кода макроса для анализа
struct FireworkAst {
    tokens: TokenStream2,
}

impl Parse for FireworkAst {
    fn parse(input: ParseStream) -> Result<Self> {
        let tokens: TokenStream2 = input.parse()?;

        Ok(FireworkAst {
            tokens
        })
    }
}

#[proc_macro]
pub fn ui(input: TokenStream) -> TokenStream {
    // Парсинг кода макроса в абстрактно синтаксическое дерево
    let ast = parse_macro_input!(input as FireworkAst);

    // Генерация раст кода, если компилятор вернул ошибку (Err) то оборачиваем
    // её в красивую ошибку компиляции
    let generated_rust_code_string = match run_firework_compiler(ast) {
        Ok(code_string) => code_string,
        Err(err_msg) => {
            let err = syn::Error::new(proc_macro2::Span::call_site(), err_msg);
            return err.to_compile_error().into();
        }
    };

    // Отправка созданной компилятором Firework строки в rustc
    let output_tokens: TokenStream2 = generated_rust_code_string
        .parse()
        .expect("FATAL: Firework compiler generated_code is invalid");

    output_tokens.into()
}
