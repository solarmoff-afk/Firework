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

pub const SPARK_UNIQUE_NAME_ERROR: &str = "\
error[FE004]: spark variable requires a unique name
   = note: spark!() must be initialized with a new variable name
   = help: use a different name that is not already in scope
   = note: for more information, see: [WORK IN PROGRESS]
";

pub const SPARK_GLOBAL_ERROR: &str = "\
error[FE005]: spark cannot be used in global or static context
   = note: spark!() is only allowed in local scope
   = help: move spark!() inside a function body
   = note: spark values are automatically globalized for screen transitions
   = note: for more information, see: [WORK IN PROGRESS]
";

pub const SPARK_MULTIPLE_ERROR: &str = "\
error[FE006]: multiple spark!() calls in the same expression are not allowed
   = note: each expression can only contain one spark!() initializer
   = help: split the expression into separate spark variable declarations
   = help: example: `let a = spark!(1); let b = spark!(2); let c = a - b;`
   = note: for more information, see: [WORK IN PROGRESS]
";

pub const WIDGET_PARSE_ERROR: &str = "\
error[FE007]: failed to parse widget macro invocation
   = note: expected syntax: `widget_name!(field: value, field: value);`
   = help: check that all fields follow the `field: value` pattern
   = help: fields must be separated by commas
   = note: for more information, see: [WORK IN PROGRESS]
";

pub const LAYOUT_PARSE_ERROR: &str = "\
error[FE008]: failed to parse layout macro invocation
   = note: layout macros expect valid Rust code, custom syntax is only available in widgets
   = help: use valid Rust expressions inside the layout block
   = note: example: `vertical! { println!(\"Hi!\"); };`
   = note: for more information, see: [WORK IN PROGRESS]
";

pub const LAYOUT_MULTIPLE_ERROR: &str = "\
error[FE009]: layout!() widget can only be used once per layout block
   = note: layout configuration has already been set in this block
   = help: remove duplicate layout!() widget calls
   = help: keep only one layout!() widget at the beginning of the block
   = note: for more information, see: [WORK IN PROGRESS]
";

pub const MACRO_BRACE_ERROR: &str = "\
error[FE010]: widget and layout macros only accept `{}` braces
   = note: using `()` or `[]` is not supported
   = help: change to curly braces: `widget_name! { ... }` or `layout_name! { ... }`
   = note: for more information, see: [WORK IN PROGRESS]
";

pub const SPARK_MUT_REQUIRED_ERROR: &str = "\
error[FE011]: cannot assign to reactive variable without `mut`
   = note: `spark!()` creates a reactive value that may be mutated
   = help: consider changing this binding to `mut`: `let mut name: Type = spark!(value);`
   = note: for more information, see: [WORK IN PROGRESS]
";

pub fn compile_error_spanned<T: quote::ToTokens>(tokens: T, msg: &str) -> Error {
    Error::new_spanned(tokens, msg)
}