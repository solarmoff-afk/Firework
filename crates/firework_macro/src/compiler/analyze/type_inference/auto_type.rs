// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use quote::ToTokens;
use syn::{Expr, Lit};

/// Хелпер который нужен чтобы угадать тип переменной на основе выражения которое
/// передаётся в spark!( ... ), нужен для повышения DX
pub fn guess_type_from_expr(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            // Целое число
            Lit::Int(i) => {
                let suffix = i.suffix();

                // По умолчанию в расте используется i32
                Some(if suffix.is_empty() {
                    "i32".to_string()
                } else {
                    suffix.to_string()
                })
            }

            // Дробное число
            Lit::Float(f) => {
                let suffix = f.suffix();

                // По умолчанию в расте используется f64 для дробных чисел
                Some(if suffix.is_empty() {
                    "f64".to_string()
                } else {
                    suffix.to_string()
                })
            }

            // Булевое
            Lit::Bool(_) => Some("bool".to_string()),

            // TODO: Нужно генерировать структуру экрана с учётом лайфтаймов
            Lit::Str(_) => Some("&'static str".to_string()),

            // Символ
            Lit::Char(_) => Some("char".to_string()),

            _ => None,
        },

        // Вызовы функций
        Expr::Call(expr_call) => {
            if let Expr::Path(path_expr) = &*expr_call.func {
                // Удаление пробелов для матча
                let path_str = path_expr.to_token_stream().to_string().replace(" ", "");

                match path_str.as_str() {
                    // Строка
                    "String::from" | "String::new" => Some("::std::string::String".to_string()),

                    // Option
                    "Some" => {
                        let inner_type = expr_call
                            .args
                            .first()
                            .and_then(guess_type_from_expr)
                            .unwrap_or_else(|| "_".to_string());

                        Some(format!("::std::option::Option<{}>", inner_type))
                    }

                    // Box
                    "Box::new" => {
                        let inner_type = expr_call
                            .args
                            .first()
                            .and_then(guess_type_from_expr)
                            .unwrap_or_else(|| "_".to_string());

                        Some(format!("::std::boxed::Box<{}>", inner_type))
                    }

                    // Arc
                    "Arc::new" => {
                        let inner_type = expr_call
                            .args
                            .first()
                            .and_then(guess_type_from_expr)
                            .unwrap_or_else(|| "_".to_string());

                        Some(format!("::std::sync::Arc<{}>", inner_type))
                    }

                    // Rc
                    "Rc::new" => {
                        let inner_type = expr_call
                            .args
                            .first()
                            .and_then(guess_type_from_expr)
                            .unwrap_or_else(|| "_".to_string());

                        Some(format!("::std::rc::Rc<{}>", inner_type))
                    }

                    // RefCell
                    "RefCell::new" => {
                        let inner_type = expr_call
                            .args
                            .first()
                            .and_then(guess_type_from_expr)
                            .unwrap_or_else(|| "_".to_string());

                        Some(format!("::std::cell::RefCell<{}>", inner_type))
                    }

                    // Mutex
                    "Mutex::new" => {
                        let inner_type = expr_call
                            .args
                            .first()
                            .and_then(guess_type_from_expr)
                            .unwrap_or_else(|| "_".to_string());

                        Some(format!("::std::sync::Mutex<{}>", inner_type))
                    }

                    // RwLock
                    "RwLock::new" => {
                        let inner_type = expr_call
                            .args
                            .first()
                            .and_then(guess_type_from_expr)
                            .unwrap_or_else(|| "_".to_string());

                        Some(format!("::std::sync::RwLock<{}>", inner_type))
                    }

                    // Если это какой-то другой вызов конструктора
                    _ => None,
                }
            } else {
                None
            }
        }

        // Конструкция структуры
        Expr::Struct(expr_struct) => {
            let struct_name = expr_struct
                .path
                .to_token_stream()
                .to_string()
                .replace(" ", "");

            Some(struct_name)
        }

        // Кортежи (10, true) -> (i32, bool)
        Expr::Tuple(expr_tuple) => {
            let mut types = Vec::new();

            for elem in &expr_tuple.elems {
                types.push(guess_type_from_expr(elem).unwrap_or_else(|| "_".to_string()));
            }

            Some(format!("({})", types.join(", ")))
        }

        // Массивы [1, 2, 3] -> [i32; 3]
        Expr::Array(expr_array) => {
            let len = expr_array.elems.len();
            let inner_type = expr_array
                .elems
                .first()
                .and_then(guess_type_from_expr)
                .unwrap_or_else(|| "_".to_string());

            Some(format!("[{}; {}]", inner_type, len))
        }

        // Если не получилось угадать тип
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use syn::parse_str;

    fn convert_to_expr(string: &str) -> Expr {
        parse_str(string).unwrap()
    }

    fn guess_type(string: &str) -> Option<String> {
        guess_type_from_expr(&convert_to_expr(string))
    }

    #[test]
    fn test_type_inference_primitive() {
        assert_eq!(guess_type("10"), Some("i32".to_string()));
        assert_eq!(guess_type("0.5"), Some("f64".to_string()));
        assert_eq!(guess_type("true"), Some("bool".to_string()));
        assert_eq!(guess_type("false"), Some("bool".to_string()));
        assert_eq!(guess_type("'a'"), Some("char".to_string()));

        // Нельзя угадать тип
        assert_eq!(guess_type("eifwepofkew"), None);
    }

    #[test]
    fn test_type_inference_with_suffix() {
        assert_eq!(guess_type("10u32"), Some("u32".to_string()));
        assert_eq!(guess_type("10.5f32"), Some("f32".to_string()));
    }

    #[test]
    fn test_type_inference_construct() {
        assert_eq!(
            guess_type("Some(10)"),
            Some("::std::option::Option<i32>".to_string())
        );
        assert_eq!(
            guess_type("Some(10u32)"),
            Some("::std::option::Option<u32>".to_string())
        );
        assert_eq!(
            guess_type("Some(Some(10u32))"),
            Some("::std::option::Option<::std::option::Option<u32>>".to_string())
        );
    }

    #[test]
    fn test_type_inference_string() {
        assert_eq!(
            guess_type("String::new()"),
            Some("::std::string::String".to_string())
        );
        assert_eq!(
            guess_type("String::from(\"Hello\")"),
            Some("::std::string::String".to_string())
        );
        assert_eq!(
            guess_type("Some(String::new())"),
            Some("::std::option::Option<::std::string::String>".to_string())
        );
        assert_eq!(guess_type("\"Hello\""), Some("&'static str".to_string()));
    }

    #[test]
    fn test_type_inference_smart_pointers() {
        assert_eq!(
            guess_type("Box::new(10)"),
            Some("::std::boxed::Box<i32>".to_string())
        );
        assert_eq!(
            guess_type("Box::new(Some(10))"),
            Some("::std::boxed::Box<::std::option::Option<i32>>".to_string())
        );
        assert_eq!(
            guess_type("Rc::new(10)"),
            Some("::std::rc::Rc<i32>".to_string())
        );
        assert_eq!(
            guess_type("Rc::new(Some(10))"),
            Some("::std::rc::Rc<::std::option::Option<i32>>".to_string())
        );
        assert_eq!(
            guess_type("Arc::new(10)"),
            Some("::std::sync::Arc<i32>".to_string())
        );
        assert_eq!(
            guess_type("Arc::new(Some(10))"),
            Some("::std::sync::Arc<::std::option::Option<i32>>".to_string())
        );
    }

    #[test]
    fn test_type_inference_struct() {
        assert_eq!(
            guess_type("MyFunc { field: 10, }"),
            Some("MyFunc".to_string())
        );
        assert_eq!(
            guess_type("MyFunc::<i32> {field: 10}"),
            Some("MyFunc::<i32>".to_string())
        );
    }
}
