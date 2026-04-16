// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use syn::Error;

/// Маркер spark можно использовать только как значение для переменной. Это нужно чтобы
/// сопоставить реактивную переменную в IR
// [UNUSED_ERROR]
// pub const SPARK_USAGE_ERROR: &str = "\
// error[FE001]: spark may only be used as a variable initializer
//   = note: spark!() creates a reactive value that must be bound to a variable
//   = help: assign the spark to a variable: `let name = spark!(value);`
//   = note: for more information, see: [WORK IN PROGRESS]
// ";

/// Нельзя затенять спарк переменную, это нужно для упрощения составления IR и кодогенерации
/// на его основе. Продолжение этого правила: FE004
pub const SPARK_SHADOWING_ERROR: &str = "\
error[FE002]: spark variables cannot be shadowed
   = note: shadowing a variable that was initialized with spark!() breaks the reactive chain
   = help: use a different variable name instead of shadowing
   = note: for more information, see: [WORK IN PROGRESS]
";

/// Спарк должен быть инициализирован с прямой аннотацией типа, но встроенный TypeChecker
/// может дать тип данных если он очевиден (Например, spark!(10)), но не если это клон
/// или другое неяное указание типа
pub const SPARK_TYPE_ERROR: &str = "\
error[FE003]: type annotations needed for spark variable
   = note: spark!() creates a reactive value that requires an explicit type
   = help: annotate the variable type: `let mut name: Type = spark!(value);`
   = help: example annotate the variable type: `let mut name: u32 = spark!(0);`
   = note: for more information, see: [WORK IN PROGRESS]
";

/// Продолжение FE002, при инициаилизации реактивной переменной нужно использовать
/// уникальное имя для этой области видимости
pub const SPARK_UNIQUE_NAME_ERROR: &str = "\
error[FE004]: spark variable requires a unique name
   = note: spark!() must be initialized with a new variable name
   = help: use a different name that is not already in scope
   = note: for more information, see: [WORK IN PROGRESS]
";

// [UNUSED_ERROR]
// pub const SPARK_GLOBAL_ERROR: &str = "\
// error[FE005]: spark cannot be used in global or static context
//   = note: spark!() is only allowed in local scope
//   = help: move spark!() inside a function body
//   = note: spark values are automatically globalized for screen transitions
//   = note: for more information, see: [WORK IN PROGRESS]
// ";

/// Запрещено использовать несколько спарков в одном выражении, это нужно для упрощения
/// генерации IR
pub const SPARK_MULTIPLE_ERROR: &str = "\
error[FE006]: multiple spark!() calls in the same expression are not allowed
   = note: each expression can only contain one spark!() initializer
   = help: split the expression into separate spark variable declarations
   = help: example: `let mut a = spark!(1); let mut b = spark!(2); let c = a - b;`
   = note: for more information, see: [WORK IN PROGRESS]
";

/// Нарушение синтаксиса Widget DSL
pub const WIDGET_PARSE_ERROR: &str = "\
error[FE007]: failed to parse widget macro invocation
   = note: expected syntax: `widget_name!(field: value, field: value);`
   = help: check that all fields follow the `field: value` pattern
   = help: fields must be separated by commas
   = note: for more information, see: [WORK IN PROGRESS]
";

/// Невалидный rust код в использовании лайаута
pub const LAYOUT_PARSE_ERROR: &str = "\
error[FE008]: failed to parse layout macro invocation
   = note: layout macros expect valid Rust code, custom syntax is only available in widgets
   = help: use valid Rust expressions inside the layout block
   = note: example: `vertical! { println!(\"Hi!\"); };`
   = note: for more information, see: [WORK IN PROGRESS]
";

/// Запрещено использовать функциональный виджет layout! несколько раз, так как нельзя
/// на ходу измененить настройки лайаута без использвания состояния
pub const LAYOUT_MULTIPLE_ERROR: &str = "\
error[FE009]: layout!() widget can only be used once per layout block
   = note: layout configuration has already been set in this block
   = help: remove duplicate layout!() widget calls
   = help: keep only one layout!() widget at the beginning of the block
   = note: for more information, see: [WORK IN PROGRESS]
";

/// Запрещено использовать () или [] в маркерах firework. Это нужно так как вызовы типа
/// rect!(margin: 123); будут неправильно обработаны syn и будут дубликаты в IR из-за
/// чего кодогенерация будет неверной
pub const MACRO_BRACE_ERROR: &str = "\
error[FE010]: widget and layout macros only accept `{}` braces
   = note: using `()` or `[]` is not supported
   = help: change to curly braces: `widget_name! { ... }` or `layout_name! { ... }`
   = note: for more information, see: [WORK IN PROGRESS]
";

/// Для изменения спарка (реактивной переменной) она должна быть мутабельной. Это правило
/// не имеет технического обоснования, нужно просто для единообразия с обычным растом
/// (чтобы нельзя было изменять имутабельную переменную если она реактивная)
pub const SPARK_MUT_REQUIRED_ERROR: &str = "\
error[FE011]: cannot assign to reactive variable without `mut`
   = note: `spark!()` creates a reactive value that may be mutated
   = help: consider changing this binding to `mut`: `let mut name: Type = spark!(value);`
   = note: for more information, see: [WORK IN PROGRESS]
";

/// Эффект должен содержать блок с логикой которая будет вызываться при изменении зависимостей
pub const EFFECT_MISSING_BODY_ERROR: &str = "\
error[FE012]: effect!() requires a body block as the last argument
   = note: effect!() tracks reactive dependencies and re-runs the body when they change
   = help: add a block `{ ... }` after the reactive variables
   = help: example: `effect!(spark1, spark2, { println!(\"changed!\"); });`
   = note: for more information, see: [WORK IN PROGRESS]
";

/// Эффект должен отслеживать хотя бы один спарк иначе он никогда не сработает
// [UNUSED_ERROR]
// pub const EFFECT_NO_SPARKS_ERROR: &str = "\
// error[FE013]: effect!() must track at least one reactive variable
//   = note: effects without reactive dependencies never trigger and are useless
//   = help: pass at least one spark variable as an argument before the body block
//   = help: example: `effect!(my_spark, { println!(\"changed!\"); });`
//   = note: for more information, see: [WORK IN PROGRESS]
// ";

/// Запрещено инициализировать спарк в ветках match или if без использования блока
pub const SPARK_BLOCK_REQUIRED_ERROR: &str = "\
error[FE014]: spark!() cannot be initialized in a simple expression arm
   = note: reactive variables require an explicit block scope `{ ... }` for lifetime tracking
   = help: wrap the branch body in curly braces and use a let binding
   = help: example: `0 => { let mut a = spark!(0); }`
   = note: for more information, see: [WORK IN PROGRESS]
";

pub fn compile_error_spanned<T: quote::ToTokens>(tokens: T, msg: &str) -> Error {
    Error::new_spanned(tokens, msg)
}
