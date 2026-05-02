// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

mod widgets;

use super::*;

fn assert_ir_equal(actual: &[FireworkStatement], expected: &[FireworkAction]) {
    assert_eq!(
        actual.len(),
        expected.len(),
        "IR length mismatch: expected {}, got {}",
        expected.len(),
        actual.len()
    );

    for (index, statement) in actual.iter().enumerate() {
        // FireworkAction содержит TokenStream который не реализует трейт для сравнения
        // поэтому нужен std::mem::discriminant
        assert!(
            std::mem::discriminant(&statement.action) == std::mem::discriminant(&expected[index]),
            "Mismatch at index {}: got {:?}, expected {:?}",
            index,
            statement.action,
            expected[index]
        );
    }
}

fn extract_ir(tokens: proc_macro2::TokenStream) -> FireworkIR {
    // let tokens_vec: Vec<_> = tokens.into_iter().collect();
    let file: File = syn::parse2(tokens.into()).unwrap();
    prepare_tokens(file, CompileFlags::new(), 0)
        .2
        .expect("IR not found")
}

fn create_initial_spark(
    name: &str,
    id: usize,
    spark_type: &str,
    expr_body: &str,
    is_mut: bool,
) -> FireworkAction {
    FireworkAction::InitialSpark {
        name: name.to_string(),
        id,
        spark_type: spark_type.to_string(),
        expr_body: expr_body.to_string(),
        is_mut,
    }
}

fn create_drop_spark(name: &str, id: usize) -> FireworkAction {
    FireworkAction::DropSpark {
        name: name.to_string(),
        id,
    }
}

fn create_reactive_block(
    block_type: FireworkReactiveBlock,
    deps: Vec<(String, usize)>,
) -> FireworkAction {
    FireworkAction::ReactiveBlock(block_type, deps, false)
}

/// Тест инициализации спарка, будет ли его инициализация в IR с нужными значениями под
/// нужным индексом
#[test]
fn test_analyze_initial_spark() {
    let tokens = quote::quote! {
        fn screen() {
            let mut a: u32 = spark!(0);
        }
    };

    let ir = extract_ir(tokens);

    let expected = [
        create_initial_spark("a", 1, "u32", "0", true),
        create_drop_spark("a", 1),
        FireworkAction::Terminator,
    ];

    assert_ir_equal(&ir.statements, &expected);
}

/// Второй тест на инициализацию, здесь проверяется что невозможность вывода типа должна
/// означачь константу "NO TYPE" и отстуствие mut также должно быть в IR
#[test]
fn test_analyze_initial_spark2() {
    let tokens = quote::quote! {
        fn screen() {
            let b = spark!(a + 10);
        }
    };

    let ir = extract_ir(tokens);

    let expected = [
        create_initial_spark("b", 1, "NO TYPE", "a + 10", false),
        create_drop_spark("b", 1),
        FireworkAction::Terminator,
    ];

    assert_ir_equal(&ir.statements, &expected);
}

/// Базовый тест реактивного блока и обновления спарка
#[test]
fn test_analyze_update_spark() {
    let tokens = quote::quote! {
        fn screen() {
            let mut a = spark!(10);

            if a == 5 {
                println!("Hello world");
            }

            a += 1;
        }
    };

    let ir = extract_ir(tokens);

    let expected = [
        create_initial_spark("a", 1, "i32", "10", true),
        create_reactive_block(
            FireworkReactiveBlock::ReactiveIf,
            vec![("a".to_string(), 1)],
        ),
        FireworkAction::DefaultCode,
        FireworkAction::ReactiveBlockTerminator,
        FireworkAction::UpdateSpark("a".to_string(), 1, None),
        create_drop_spark("a", 1),
        FireworkAction::Terminator,
    ];

    assert_ir_equal(&ir.statements, &expected);
}

/// Тестирует что обновление спарка через поле добавляет UpdateSpark в IR
#[test]
fn test_analyze_update_spark_field() {
    let tokens = quote::quote! {
        fn screen() {
            let mut a = spark!( a.clone() );
            a.field.my_field.sub_field += 1;
        }
    };

    let ir = extract_ir(tokens);

    let expected = [
        create_initial_spark("a", 1, "NO TYPE", "a . clone ()", true),
        FireworkAction::UpdateSpark("a".to_string(), 1, None),
        create_drop_spark("a", 1),
        FireworkAction::Terminator,
    ];

    assert_ir_equal(&ir.statements, &expected);
}

/// Тест обработки вложенных спарков, лайфтайм менеджер должен правильно поставить
/// DropSpark чтобы вернуть владение над данными реактивной переменной в статику
#[test]
fn test_analyze_lifetime() {
    let tokens = quote::quote! {
        fn screen() {
            let mut a = spark!(10);

            if a == 5 {
                let b: f32 = spark!(10.0);
                println!("Hello world");
            }
        }
    };

    let ir = extract_ir(tokens);

    let expected = [
        create_initial_spark("a", 1, "i32", "10", true),
        create_reactive_block(
            FireworkReactiveBlock::ReactiveIf,
            vec![("a".to_string(), 1)],
        ),
        create_initial_spark("b", 2, "f32", "10.0", false),
        FireworkAction::DefaultCode,
        create_drop_spark("b", 2),
        FireworkAction::ReactiveBlockTerminator,
        create_drop_spark("a", 1),
        FireworkAction::Terminator,
    ];

    assert_ir_equal(&ir.statements, &expected);
}

/// Проверяет что поведение обычных блоков (не реактивных) не отличается от реактивных
/// в анализе времён жизни
#[test]
fn test_analyze_lifetime_base() {
    let tokens = quote::quote! {
        fn screen() {
            let mut a = spark!(10);

            if 5 == 5 {
                let b: f32 = spark!(10.0);
                println!("Hello world");
            }
        }
    };

    let ir = extract_ir(tokens);

    let expected = [
        create_initial_spark("a", 1, "i32", "10", true),
        FireworkAction::DefaultCode, // if 5 == 5 {}
        create_initial_spark("b", 2, "f32", "10.0", false),
        FireworkAction::DefaultCode,
        create_drop_spark("b", 2),
        FireworkAction::ReactiveBlockTerminator,
        create_drop_spark("a", 1),
        FireworkAction::Terminator,
    ];

    assert_ir_equal(&ir.statements, &expected);
}

/// Тест на синтаксис computed спарков, UpdateSpark с спарками в выражении должно
/// быть обёрнуто эффектом, а также реактивные блоки внутри реактивных блоков
/// должны быть обработаны нормально
#[test]
fn test_analyze_compute_spark() {
    let tokens = quote::quote! {
        fn screen() {
            let mut a = spark!(10);

            if a == 5 {
                let b: f32 = spark!(10.0);
                a = b * 2;
            }
        }
    };

    let ir = extract_ir(tokens);

    let expected = [
        create_initial_spark("a", 1, "i32", "10", true),
        create_reactive_block(
            FireworkReactiveBlock::ReactiveIf,
            vec![("a".to_string(), 1)],
        ),
        create_initial_spark("b", 2, "f32", "10.0", false),
        create_reactive_block(FireworkReactiveBlock::Effect, vec![("b".to_string(), 2)]),
        FireworkAction::UpdateSpark("a".to_string(), 1, None),
        FireworkAction::ReactiveBlockTerminator,
        create_drop_spark("b", 2),
        FireworkAction::ReactiveBlockTerminator,
        create_drop_spark("a", 1),
        FireworkAction::Terminator,
    ];

    assert_ir_equal(&ir.statements, &expected);
}

/// Тест на синтаксис эффекта
#[test]
fn test_analyze_effect() {
    let tokens = quote::quote! {
        fn screen() {
            let mut a = spark!(10);

            effect!(a, {
                a += 1;
                println!("Hello world");
            });

            a += 1;
        }
    };

    let ir = extract_ir(tokens);

    let expected = [
        create_initial_spark("a", 1, "i32", "10", true),
        create_reactive_block(FireworkReactiveBlock::Effect, vec![("a".to_string(), 1)]),
        FireworkAction::UpdateSpark("a".to_string(), 1, None),
        FireworkAction::DefaultCode,
        FireworkAction::ReactiveBlockTerminator,
        FireworkAction::UpdateSpark("a".to_string(), 1, None),
        create_drop_spark("a", 1),
        FireworkAction::Terminator,
    ];

    assert_ir_equal(&ir.statements, &expected);
}

#[test]
fn test_closure_return_does_not_drop_outer_sparks() {
    let tokens = quote::quote! {
        fn screen() {
            let mut a = spark!(10);

            let closure = || {
                return;
            };

            closure();

            a += 1;
        }
    };

    let ir = extract_ir(tokens);

    let expected = [
        create_initial_spark("a", 1, "i32", "10", true),
        FireworkAction::DefaultCode, // let closure = || { ... };
        FireworkAction::DefaultCode, // closure();
        FireworkAction::UpdateSpark("a".to_string(), 1, None),
        create_drop_spark("a", 1),
        FireworkAction::Terminator,
    ];

    assert_ir_equal(&ir.statements, &expected);
}

fn create_effect_test_pattern() -> Vec<FireworkAction> {
    vec![
        create_initial_spark("a", 1, "i32", "10", true),
        create_reactive_block(FireworkReactiveBlock::Effect, vec![("a".to_string(), 1)]),
        FireworkAction::UpdateSpark("a".to_string(), 1, None),
        FireworkAction::DefaultCode,
        FireworkAction::ReactiveBlockTerminator,
        FireworkAction::UpdateSpark("a".to_string(), 1, None),
        create_drop_spark("a", 1),
        FireworkAction::Terminator,
    ]
}

/// Тест на анализ нескольких экранов в одном макросе
#[test]
fn test_analyze_effect_multifunc() {
    let tokens = quote::quote! {
        fn screen() {
            let mut a = spark!(10);

            effect!(a, {
                a += 1;
                println!("Hello world");
            });

            a += 1;
        }

        fn screen2() {
            let mut a = spark!(10);

            effect!(a, {
                a += 1;
                println!("Hello world");
            });

            a += 1;
        }
    };

    let ir = extract_ir(tokens);
    let pattern = create_effect_test_pattern();

    let mut expected = Vec::new();
    expected.extend_from_slice(&pattern);
    expected.extend_from_slice(&pattern);

    assert_ir_equal(&ir.statements, &expected);
}

macro_rules! ir_sequence {
    ($($action:expr),* $(,)?) => {
        vec![$($action),*]
    };
}

#[test]
fn test_with_macro_helper() {
    use FireworkAction::*;

    let expected = ir_sequence![
        InitialSpark {
            name: "x".to_string(),
            id: 1,
            spark_type: "i32".to_string(),
            expr_body: "42".to_string(),
            is_mut: true
        },
        Terminator,
    ];

    assert_eq!(expected.len(), 2);
}
