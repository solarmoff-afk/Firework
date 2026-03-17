// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

extern crate proc_macro;

mod compiler;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use compiler::*;

// TODO:
//  - Начать писать генератор FIREWORK-IR в prepare 

// Система id нужна для того чтобы построить уникальную структуру в статической памяти
// и её экземпляр
static BLOCK_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Структура абстрактного синтаксического дерева. Здесь хранятся токены после
/// парсинга кода макроса для анализа
struct FireworkAst {
    tokens: TokenStream2,
}

impl Parse for FireworkAst {
    fn parse(input: ParseStream) -> Result<Self> {
        let tokens: TokenStream2 = input.parse()?;

        Ok(FireworkAst {
            tokens
        })
    }
}

#[proc_macro]
pub fn ui(input: TokenStream) -> TokenStream {
    let id = BLOCK_COUNTER.fetch_add(1, Ordering::Relaxed);

    // Парсинг кода макроса в абстрактно синтаксическое дерево
    let ast = parse_macro_input!(input as FireworkAst);

    // Генерация раст кода, если компилятор вернул ошибку (Err) то оборачиваем
    // её в красивую ошибку компиляции
    // let generated_rust_code_string = match run_firework_compiler(ast, id) {
    //    Ok(code_string) => code_string,
    //    Err(err_msg) => {
    //        let err = syn::Error::new(proc_macro2::Span::call_site(), err_msg);
    //        return err.to_compile_error().into();
    //    }
    // };

    // Отправка созданной компилятором Firework строки в rustc
    // let output_tokens: TokenStream2 = generated_rust_code_string
    //    .parse()
    //    .expect("FATAL: Firework compiler generated_code is invalid");

    let (token_stream, error_tokens) = run_firework_compiler_temp(ast, id);
    
    let mut output: proc_macro2::TokenStream = token_stream.into();
    
    // Если есть ошибки компиляции - добавляем их к выходному потоку
    // Каждая ошибка уже содержит правильный спан через compile_error! макрос
    if let Some(err_tokens) = error_tokens {
        output.extend(err_tokens);
    }
    
    output.into()
}

// Заглушка
#[proc_macro_attribute]
pub fn component(_args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    input
}
