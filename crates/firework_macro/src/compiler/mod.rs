// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub mod flags;

mod analyze;
mod error;
mod codegen;

use analyze::prepare_tokens;
use codegen::transform::CodegenVisitor;
use flags::CompileFlags;

use proc_macro2::TokenStream;
use syn::visit_mut::VisitMut;
use syn::File;

#[cfg(feature = "debug_output")]
use syn::parse_str;

#[cfg(feature = "debug_output")]
use prettyplease::unparse;

use crate::FireworkAst;

pub fn run_firework_compiler(
    ast: FireworkAst,
    flags: CompileFlags,
    id: u64
) -> (TokenStream, Option<TokenStream>) {
    let token_stream: TokenStream = ast.tokens.into();
    let mut file: File = syn::parse2(token_stream).unwrap();

    let output = prepare_tokens(file.clone(), flags, id);

    if let Some(mut ir) = output.2 && output.1.is_none() {
        let mut visitor = CodegenVisitor::new(&mut ir);
        visitor.set_flags(flags);
        visitor.visit_file_mut(&mut file);

        let mut codegen_output = quote::quote! {
            #file
        };

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
