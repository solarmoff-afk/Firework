// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

mod analyze2;
mod widgets;
mod utils;
mod error;
mod codegen;

use analyze2::prepare_tokens;
use codegen::generator::CodeGen;

use proc_macro2::TokenTree;
use quote::quote;

use crate::FireworkAst;

pub use error::*;

pub fn _run_firework_compiler(ast: FireworkAst, id: u64) -> Result<String, String> {
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

pub fn run_firework_compiler_temp(ast: FireworkAst, _id: u64) -> (proc_macro2::TokenStream, Option<proc_macro2::TokenStream>) {
    // Компилятор должен в любом случае вернуть заглушки чтобы не было ошибок с тем
    // что функция экрана не найдена в обоасти видимости. prepare_tokens генерирует
    // эти заглушки на случай если компиляция упадёт

    let tokens: Vec<TokenTree> = ast.tokens.clone().into_iter().collect();
    let output = prepare_tokens(tokens.clone());

    if let Some(ir) = output.2 {
        let mut codegen = CodeGen::new(ir);
        codegen.run(tokens);
    }

    (output.0, output.1)
}
