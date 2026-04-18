// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

mod analyze;
mod error;
mod codegen;

use analyze::prepare_tokens;
use codegen::transform::CodegenVisitor;

use proc_macro2::TokenStream;
use syn::visit_mut::VisitMut;

#[cfg(feature = "debug_output")]
use syn::{File, parse_str};

#[cfg(feature = "debug_output")]
use prettyplease::unparse;

use crate::FireworkAst;

pub fn run_firework_compiler(ast: FireworkAst, id: u64) -> (proc_macro2::TokenStream, Option<proc_macro2::TokenStream>) {
    let token_stream: TokenStream = ast.tokens.into();
    let mut file: syn::File = syn::parse2(token_stream).unwrap();

    let output = prepare_tokens(file.clone(), id);

    if let Some(mut ir) = output.2 {
        let mut visitor = CodegenVisitor::new(&mut ir);
        visitor.visit_file_mut(&mut file);

        let mut codegen_output = quote::quote! { #file };

        #[cfg(feature = "debug_output")]
        {
            let mut codegen_string = codegen_output.to_string();
            println!("{}", codegen_string);

            let syntax_tree: File = parse_str(&codegen_string).unwrap();
            codegen_string = unparse(&syntax_tree);

            println!("{}", codegen_string);
        }

        codegen_output.extend(output.0);

        return (codegen_output, output.1);
    }

    (output.0, output.1)
}
