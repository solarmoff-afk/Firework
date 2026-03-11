// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use syn::Error;

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

pub const SPARK_TYPE_ERROR: &str = "\
error[FE003]: type annotations needed for spark variable
   = note: spark!() creates a reactive value that requires an explicit type
   = help: annotate the variable type: `let name: Type = spark!(value);`
   = help: example annotate the variable type: `let name: u32 = spark!(0);`
   = note: for more information, see: [WORK IN PROGRESS]
";

pub fn compile_error_spanned<T: quote::ToTokens>(tokens: T, msg: &str) -> Error {
    Error::new_spanned(tokens, msg)
}
