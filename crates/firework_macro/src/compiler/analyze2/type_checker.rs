// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use syn::{Expr, Lit};
use quote::ToTokens;

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
            },

            // Дробное число
            Lit::Float(f) => {
                let suffix = f.suffix();
                
                // По умолчанию в расте используется f64 для дробных чисел
                Some(if suffix.is_empty() {
                    "f64".to_string()
                } else {
                    suffix.to_string()
                })
            },

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
                    "String::from" | "String::new" => Some("String".to_string()),
                    
                    // Option
                    "Some" => { 
                        let inner_type = expr_call.args.first()
                            .and_then(guess_type_from_expr)
                            .unwrap_or_else(|| "_".to_string());

                        Some(format!("Option<{}>", inner_type))
                    }
                    
                    // Box
                    "Box::new" => {
                        let inner_type = expr_call.args.first()
                            .and_then(guess_type_from_expr)
                            .unwrap_or_else(|| "_".to_string());
                        
                        Some(format!("Box<{}>", inner_type))
                    }
                    
                    // Если это какой-то другой вызов конструктора
                    _ => None,
                }
            } else {
                None
            }
        },

        // Конструкция структуры
        Expr::Struct(expr_struct) => {
            let struct_name = expr_struct.path.to_token_stream().to_string().replace(" ", "");
            
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
            let inner_type = expr_array.elems.first()
                .and_then(guess_type_from_expr)
                .unwrap_or_else(|| "_".to_string());
            
            Some(format!("[{}; {}]", inner_type, len))
        }

        // Если не получилось угадать тип
        _ => None,
    }
}
