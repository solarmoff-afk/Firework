// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

mod prepare;
mod widgets;

use prepare::prepare_tokens;
use proc_macro2::TokenTree;
use quote::quote;

use crate::FireworkAst;

pub fn run_firework_compiler(ast: FireworkAst, id: u64) -> std::result::Result<String, String> {
    {
        let tokens: Vec<TokenTree> = ast.tokens.clone().into_iter().collect();
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

pub fn run_firework_compiler_temp(ast: FireworkAst, id: u64) ->std::result::Result<proc_macro2::TokenStream, String> {
    let tokens: Vec<TokenTree> = ast.tokens.clone().into_iter().collect();
    Ok(prepare_tokens(tokens)) 
}
