// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

extern crate proc_macro;

mod compiler;

use compiler::flags::{CompileFlags, CompileType};
use compiler::*;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use std::sync::atomic::{AtomicU64, Ordering};
use syn::parse::{Parse, ParseStream};
use syn::{Result, parse_macro_input};

static BLOCK_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Структура абстрактного синтаксического дерева. Здесь хранятся токены после
/// парсинга кода макроса для анализа
struct FireworkAst {
    tokens: TokenStream2,
}

impl Parse for FireworkAst {
    fn parse(input: ParseStream) -> Result<Self> {
        let tokens: TokenStream2 = input.parse()?;

        Ok(FireworkAst { tokens })
    }
}

#[proc_macro]
pub fn shared(input: TokenStream) -> TokenStream {
    process_macro(input, CompileType::Shared, false)
}

#[proc_macro]
pub fn component(input: TokenStream) -> TokenStream {
    process_macro(input, CompileType::Component, false)
}

#[proc_macro]
pub fn ui_block(input: TokenStream) -> TokenStream {
    process_macro(input, CompileType::Screen, true)
}

#[proc_macro_attribute]
pub fn ui(_args: proc_macro::TokenStream, input: TokenStream) -> TokenStream {
    process_macro(input, CompileType::Screen, true)
}

fn process_macro(input: TokenStream, compile_type: CompileType, use_counter: bool) -> TokenStream {
    let ast = parse_macro_input!(input as FireworkAst);

    let flags = CompileFlags { compile_type };

    let id = if use_counter {
        BLOCK_COUNTER.fetch_add(1, Ordering::Relaxed)
    } else {
        0
    };

    let (token_stream, error_tokens) = run_firework_compiler(ast, flags, id);

    let mut output: proc_macro2::TokenStream = token_stream.into();

    // Если есть ошибки компиляции - добавляем их к выходному потоку
    // Каждая ошибка уже содержит правильный спан через compile_error! макрос
    if let Some(err_tokens) = error_tokens {
        output.extend(err_tokens);
    }

    output.into()
}

#[proc_macro_attribute]
pub fn effect(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    input
}
