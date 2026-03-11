// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use syn::Error;
use proc_macro2::Span;

pub const SPARK_USAGE_ERROR: &str = "\
error[FE001]: spark may only be used as a variable initializer
   = note: spark!() creates a reactive value that must be bound to a variable
   = help: assign the spark to a variable: `let name = spark!(value);`
   = note: for more information, see: [WORK IN PROGRESS]
";

pub const SPARK_SHADOWING_ERROR: &str = "\
error[FE002]: spark variables cannot be shadowed
   = note: shadowing a variable that was initialized with spark!() breaks the reactive chain
   = help: use a different variable name instead of shadowing
   = note: for more information, see: [WORK IN PROGRESS]
";

pub fn compile_error(msg: &str) -> Error {
    Error::new(Span::call_site(), msg)
}

pub fn compile_error_spanned<T: quote::ToTokens>(tokens: T, msg: &str) -> Error {
    Error::new_spanned(tokens, msg)
}
