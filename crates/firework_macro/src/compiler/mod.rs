// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

mod analyze;
mod utils;
mod error;
mod codegen;

use analyze::prepare_tokens;
use codegen::transform::CodegenVisitor;

use proc_macro2::TokenTree;
use syn::visit_mut::VisitMut;

#[cfg(feature = "debug_output")]
use syn::{File, parse_str};

#[cfg(feature = "debug_output")]
use prettyplease::unparse;

use crate::FireworkAst;

pub fn run_firework_compiler(ast: FireworkAst, id: u64) -> (proc_macro2::TokenStream, Option<proc_macro2::TokenStream>) {
    // Компилятор должен в любом случае вернуть заглушки чтобы не было ошибок с тем
    // что функция экрана не найдена в обоасти видимости. prepare_tokens генерирует
    // эти заглушки на случай если компиляция упадёт
    let tokens: Vec<TokenTree> = ast.tokens.clone().into_iter().collect();
    let output = prepare_tokens(tokens, id);

    if let Some(mut ir) = output.2 {
        let token_stream: proc_macro2::TokenStream = ast.tokens.into();
        let mut file: syn::File = syn::parse2(token_stream).unwrap();

        let mut visitor = CodegenVisitor::new(&mut ir);
        visitor.visit_file_mut(&mut file);

        let codegen_output = quote::quote! { #file };

        #[cfg(feature = "debug_output")]
        {
            let mut codegen_string = codegen_output.to_string();
            println!("{}", codegen_string);

            let syntax_tree: File = parse_str(&codegen_string).unwrap();
            codegen_string = unparse(&syntax_tree);

            println!("{}", codegen_string);
        }

        return (codegen_output, output.1);
    }

    (output.0, output.1)
}
