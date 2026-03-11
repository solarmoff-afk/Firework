// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

mod prepare;
mod widgets;
mod error;

use prepare::prepare_tokens;
use proc_macro2::TokenTree;
use quote::quote;

use crate::FireworkAst;

pub use error::*;

/// Обёртка над Result для удобства изменения
pub type CompileResult<T> = std::result::Result<T, syn::Error>;

pub fn run_firework_compiler(ast: FireworkAst, id: u64) -> Result<String, String> {
    {
        let tokens: Vec<TokenTree> = ast.tokens.clone().into_iter().collect();
        
        // Компилятор должен в любом случае вернуть заглушки чтобы не было ошибок с тем
        // что функция экрана не найдена в обоасти видимости. prepare_tokens генерирует
        // эти заглушки на случай если компиляция упадёт
        prepare_tokens(tokens);
    }

    let _raw_input_string = ast.tokens.to_string();
 
    let generated_code = quote! {
        {
            const SCREEN_ID: u64 = #id;

            println!("Firework test");
        }
    };

    Ok(generated_code.to_string())
}

pub fn run_firework_compiler_temp(ast: FireworkAst, id: u64) -> (proc_macro2::TokenStream, Option<String>) {
    // Компилятор должен в любом случае вернуть заглушки чтобы не было ошибок с тем
    // что функция экрана не найдена в обоасти видимости. prepare_tokens генерирует
    // эти заглушки на случай если компиляция упадёт

    let tokens: Vec<TokenTree> = ast.tokens.clone().into_iter().collect();
    prepare_tokens(tokens) 
}
