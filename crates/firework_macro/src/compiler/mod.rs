// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub mod flags;

mod analyze;
mod codegen;
mod error;

use analyze::prepare_tokens;
use codegen::lower::visitors_mut::LowerVisitor;
use codegen::transform::CodegenVisitor;
use flags::CompileFlags;

use proc_macro2::TokenStream;
use syn::File;
use syn::visit_mut::VisitMut;

#[cfg(feature = "debug_output")]
use syn::parse_str;

#[cfg(feature = "debug_output")]
use prettyplease::unparse;

#[cfg(feature = "trace")]
use tracing::*;

#[cfg(feature = "trace")]
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;

#[cfg(feature = "trace")]
use tracing_subscriber::util::SubscriberInitExt;

use crate::FireworkAst;

pub fn run_firework_compiler(
    ast: FireworkAst,
    flags: CompileFlags,
    id: u64,
) -> (TokenStream, Option<TokenStream>) {
    #[cfg(feature = "trace")]
    let (_chrome_guard, _sub_guard) = {
        let (chrome_layer, chrome_guard) = tracing_chrome::ChromeLayerBuilder::new()
            .file(format!("target/trace_{}.json", id))
            .include_args(true)
            .build();

        let sub_guard = tracing_subscriber::registry()
            .with(chrome_layer)
            .set_default();

        info!(compiler_id = id, "Firework Compiler started");

        (chrome_guard, sub_guard)
    };

    let token_stream: TokenStream = ast.tokens;
    let mut file: File = syn::parse2(token_stream).unwrap();

    let output = {
        #[cfg(feature = "trace")]
        let _span = info_span!("analyze::prepare_tokens").entered();

        prepare_tokens(file.clone(), flags, id)
    };

    if let Some(mut ir) = output.2
        && output.1.is_none()
    {
        {
            #[cfg(feature = "trace")]
            let _span = info_span!("codegen::lower").entered();

            let mut visitor = LowerVisitor::new(&mut ir, flags);
            visitor.visit_file_mut(&mut file);
        }

        {
            #[cfg(feature = "trace")]
            let _span = info_span!("codegen::transform").entered();

            let mut visitor = CodegenVisitor::new(&mut ir);
            visitor.set_flags(flags);
            visitor.visit_file_mut(&mut file);
        }

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
