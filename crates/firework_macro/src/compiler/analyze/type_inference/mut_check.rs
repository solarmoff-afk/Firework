// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

/// Метод для проверки явлется ли метод конкретного типа мутабельным для проверки
/// нужно ли пометить этот стейтемент семантической меткой UpdateSpark (изменение
/// реактивной переменной)
pub fn is_mutable_method(type_name: &str, method: &str) -> bool {
    let type_name = type_name.trim();
    let method = method.trim();

    parse_nested_types(type_name).iter().any(|t| check_type_mutable(t, method))
}

fn parse_nested_types(type_name: &str) -> Vec<String> {
    let mut types = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    
    for type_char in type_name.chars() {
        match type_char {
            '<' => {
                if depth == 0 {
                    if !current.is_empty() {
                        types.push(current.trim().to_string());
                        current.clear();
                    }
                } else {
                    current.push(type_char);
                }

                depth += 1;
            },

            '>' => {
                depth -= 1;
                if depth == 0 {
                    if !current.is_empty() {
                        types.push(current.trim().to_string());
                        current.clear();
                    }
                } else {
                    current.push(type_char);
                }
            },

            ',' => {
                if depth == 1 {
                    if !current.is_empty() {
                        types.push(current.trim().to_string());
                        current.clear();
                    }
                } else {
                    current.push(type_char);
                }
            },

            _ => {
                if depth > 0 || !type_char.is_whitespace() {
                    current.push(type_char);
                }
            },
        }
    }
    
    if !current.is_empty() {
        types.push(current.trim().to_string());
    }
    
    types
}

fn check_type_mutable(type_name: &str, method: &str) -> bool {
    let base_type = type_name.split('<').next().unwrap_or(type_name).trim();
    
    match base_type {
        // У примитивов нет мутабельных методов
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" |
        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" |
        "f32" | "f64" | "bool" | "char" | "str" => {
            let mutable_methods: [&str; 0] = [];
            mutable_methods.contains(&method)
        },

        // Строка
        "String" => {
            let mutable_methods = [
                "push", "push_str", "insert", "insert_str", "pop", "truncate", 
                "clear", "remove", "retain", "drain", "replace_range", "reserve", 
                "reserve_exact", "shrink_to_fit", "shrink_to", "into_bytes"
            ];

            mutable_methods.contains(&method)
        },
        
        // Вектор
        "Vec" => {
            let mutable_methods = [
                "push", "pop", "insert", "remove", "swap_remove", "truncate", 
                "clear", "append", "drain", "retain", "dedup", "sort", "sort_by", 
                "sort_by_key", "sort_unstable", "sort_unstable_by", 
                "sort_unstable_by_key", "reverse", "resize", "resize_with", 
                "extend", "extend_from_slice", "splice", "split_off", "reserve", 
                "reserve_exact", "shrink_to_fit", "shrink_to", "into_boxed_slice"
            ];

            mutable_methods.contains(&method)
        },

        // Хэшмап
        "HashMap" => {
            let mutable_methods = [
                "insert", "remove", "remove_entry", "clear", "retain", "drain",
                "extend", "entry", "reserve", "shrink_to_fit", "shrink_to"
            ];

            mutable_methods.contains(&method)
        },
        
        // Ъэшсэт
        "HashSet" => {
            let mutable_methods = [
                "insert", "remove", "clear", "retain", "drain", "extend",
                "replace", "reserve", "shrink_to_fit", "shrink_to"
            ];

            mutable_methods.contains(&method)
        },
        
        "BTreeMap" => {
            let mutable_methods = [
                "insert", "remove", "remove_entry", "clear", "retain", "drain",
                "extend", "entry", "split_off", "append"
            ];

            mutable_methods.contains(&method)
        },
        
        "BTreeSet" => {
            let mutable_methods = [
                "insert", "remove", "clear", "retain", "drain", "extend",
                "replace", "split_off", "append"
            ];

            mutable_methods.contains(&method)
        },
        
        "VecDeque" => {
            let mutable_methods = [
                "push_front", "push_back", "pop_front", "pop_back", "insert", 
                "remove", "swap_remove_front", "swap_remove_back", "truncate", 
                "clear", "retain", "drain", "extend", "reserve", "reserve_exact", 
                "shrink_to_fit", "shrink_to", "rotate_left", "rotate_right", 
                "make_contiguous"
            ];

            mutable_methods.contains(&method)
        },
        
        "LinkedList" => {
            let mutable_methods = [
                "push_front", "push_back", "pop_front", "pop_back", "clear", 
                "append", "split_off", "extend", "drain_filter"
            ];

            mutable_methods.contains(&method)
        },
        
        "BinaryHeap" => {
            let mutable_methods = [
                "push", "pop", "clear", "extend", "reserve", "reserve_exact",
                "shrink_to_fit", "shrink_to", "into_sorted_vec"
            ];

            mutable_methods.contains(&method)
        },
        
        // Option
        "Option" => {
            let mutable_methods = [
                "take", "replace"
            ];

            mutable_methods.contains(&method)
        },
        
        // Result
        "Result" => {
            let mutable_methods: [&str; 0] = [];
            mutable_methods.contains(&method)
        },

        // Умный указатель бокс для данных на куче
        "Box" => {
            let box_mutable_methods = [
                // Пустота, у бокса нет мутабельных методов
            ];
            
            if box_mutable_methods.contains(&method) {
                true
            } else {
                if let Some(inner) = type_name.split('<').nth(1).and_then(|s| s.split('>').next()) {
                    parse_nested_types(inner).iter().any(|t| check_type_mutable(t, method))
                } else {
                    false
                }
            }
        },
        
        // Уиные указали (кроме Box)

        "Rc" => { 
            let rc_mutable_methods: [&str; 0] = [];
            
            if rc_mutable_methods.contains(&method) {
                true
            } else {
                false
            }
        },
        
        "Arc" => {
            let arc_mutable_methods: [&str; 0] = [];
            
            if arc_mutable_methods.contains(&method) {
                true
            } else {
                false
            }
        },
        
        "Cell" => {
            let mutable_methods = [
                "set", "swap", "replace", "take", "into_inner", "get_mut"
            ];

            mutable_methods.contains(&method)
        },
        
        "RefCell" => {
            let mutable_methods = [
                "borrow_mut", "replace", "swap", "replace_with", "take", 
                "into_inner", "get_mut"
            ];

            mutable_methods.contains(&method)
        },
        
        "Mutex" => {
            let mutable_methods = [
                "lock", "get_mut", "into_inner"
            ];

            mutable_methods.contains(&method)
        },
        
        "RwLock" => {
            let mutable_methods = [
                "write", "get_mut", "into_inner"
            ];
            mutable_methods.contains(&method)
        },
       
        // Если это не стандартный тип раста то считаем имутабельным любой метод потому-что
        // определить нормально поля для кастомных типов либо типов из других крейтов
        // невозможно
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_mut_primitive() {
        assert_eq!(is_mutable_method("i32", "push"), false);
        assert_eq!(is_mutable_method("i64", "push"), false);
        assert_eq!(is_mutable_method("f32", "push"), false);
        assert_eq!(is_mutable_method("f64", "push"), false);
        assert_eq!(is_mutable_method("u32", "push"), false);
    }

    #[test]
    fn test_check_mut_option() {
        assert_eq!(is_mutable_method("Option<i32>", "take"), true); 
    }

    #[test]
    fn test_check_mut_containers() {
        assert_eq!(is_mutable_method("Vec<i32>", "push"), true);
        assert_eq!(is_mutable_method("Vec<i32>", "pop"), true);
        assert_eq!(is_mutable_method("Vec<i32>", "remove"), true);
        assert_eq!(is_mutable_method("Vec<i32>", "sort"), true);
        assert_eq!(is_mutable_method("Vec<i32>", "last"), false);
        assert_eq!(is_mutable_method("Vec<i32>", "iter"), false);

        // String 
        assert_eq!(is_mutable_method("String", "push"), true);
        assert_eq!(is_mutable_method("String", "push_str"), true);
        assert_eq!(is_mutable_method("String", "as_str"), false);
    }

    #[test]
    fn test_check_mut_box_deref() {
        assert_eq!(is_mutable_method("Box<Vec<i32>>", "push"), true);
        assert_eq!(is_mutable_method("Box<String>", "push_str"), true);
        assert_eq!(is_mutable_method("Rc<Vec<i32>>", "push"), true);
        assert_eq!(is_mutable_method("Rc<String>", "push_str"), true);
        assert_eq!(is_mutable_method("Arc<Vec<i32>>", "push"), true);
        assert_eq!(is_mutable_method("Arc<String>", "push_str"), true);
    }
}
