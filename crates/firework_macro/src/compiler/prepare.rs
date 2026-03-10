// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use proc_macro2::TokenTree;
use proc_macro2::Delimiter;
use syn::{Stmt, token::Brace};
use quote::ToTokens;

pub fn prepare_tokens(tokens: Vec<TokenTree>) { 
    let token_stream: proc_macro2::TokenStream = tokens.clone().into_iter().collect();

    let parser = |input: syn::parse::ParseStream| {
        let mut stmts = Vec::new();
        
        while !input.is_empty() {
            stmts.push(input.parse::<syn::Stmt>()?);
        }
        
        Ok(stmts)
    };
    
    let stmts: Vec<Stmt> = syn::parse::Parser::parse2(parser, token_stream)
        .expect("Failed to parse statements");

    parse_stmts(stmts);
}

fn parse_stmts(statements: Vec<Stmt>) {
    for statement in statements {
        // println!("STATEMENT:");
        // println!("{:#?}", statement);

        match statement {
            Stmt::Local(local) => {
                println!("Local");

                parse_local(local);
            },
            
            Stmt::Item(item) => {
                println!("Item");
            },

            Stmt::Expr(expr, semi) => {
                println!("expr");
            },

            Stmt::Macro(mac) => {
                println!("Macro");
            },
        };
    }
}

fn parse_local(local: syn::Local) {
    parse_pat(local.pat);
    
}

fn parse_pat(pat: syn::Pat) {
    match pat {
        syn::Pat::Ident(ident) => {
            let is_mut = ident.mutability.is_some();
            let is_ref = ident.by_ref.is_some();
            let name = ident.ident;

            println!("Let: is_mut: {}, is_ref: {}, name: {}", is_mut, is_ref, name);
        },

        syn::Pat::Type(pat_type) => {
            let type_str = pat_type.ty.to_token_stream().to_string();
            println!("Type: {}", type_str);

            parse_pat(*pat_type.pat);
        },

        syn::Pat::Tuple(pat_tuple) => {
            for element in pat_tuple.elems.iter() {
                parse_pat(element.clone());
            }
        },

        syn::Pat::Struct(pat_struct) => {
            for field in pat_struct.fields.iter() {
                parse_pat(*field.pat.clone());
            }
        },

        syn::Pat::Slice(pat_slice) => {
            for (index, element) in pat_slice.elems.iter().enumerate() {
                parse_pat(element.clone());
            }
        },

        syn::Pat::Or(pat_or) => {
            for (index, case) in pat_or.cases.iter().enumerate() {
                parse_pat(case.clone());
            }
        },

        _ => {},
    };
}
