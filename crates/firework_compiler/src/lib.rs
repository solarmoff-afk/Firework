// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

mod compiler;

pub use compiler::flags::CompileType;

use compiler::flags::CompileFlags;
use compiler::*;
use proc_macro2::TokenStream as TokenStream2;
use std::sync::atomic::{AtomicU64, Ordering};

static BLOCK_COUNTER: AtomicU64 = AtomicU64::new(1);

struct FireworkAst {
    tokens: TokenStream2,
}

pub fn process_compile(
    input: TokenStream2,
    compile_type: CompileType,
    use_counter: bool,
) -> TokenStream2 {
    let ast = FireworkAst { tokens: input };

    let flags = CompileFlags { compile_type };

    let id = if use_counter {
        BLOCK_COUNTER.fetch_add(1, Ordering::Relaxed)
    } else {
        0
    };

    let (token_stream, error_tokens) = run_firework_compiler(ast, flags, id);

    let mut output: TokenStream2 = token_stream;

    // Если есть ошибки компиляции - добавляем их к выходному потоку
    // Каждая ошибка уже содержит правильный спан через compile_error! макрос
    if let Some(err_tokens) = error_tokens {
        output.extend(err_tokens);
    }

    output
}
