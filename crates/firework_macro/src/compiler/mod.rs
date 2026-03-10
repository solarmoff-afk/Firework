// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

mod prepare;

use prepare::prepare_tokens;
use proc_macro2::TokenTree;

use crate::FireworkAst;

pub fn run_firework_compiler(ast: FireworkAst) -> std::result::Result<String, String> {
    {
        let tokens: Vec<TokenTree> = ast.tokens.clone().into_iter().collect();
        prepare_tokens(tokens, 0);
    }

    let _raw_input_string = ast.tokens.to_string();
 
    let generated_code = r#"
        { 
            println!("Firework test");
        }
    "#;

    Ok(generated_code.to_string())
}
