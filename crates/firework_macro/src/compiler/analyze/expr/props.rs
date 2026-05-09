// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use syn::visit::Visit;
use syn::*;
use quote::ToTokens;

/// Ищет пропсы в выражении, создаётся из ссылки с вектором пропсов и ссылки куда будут
/// записаны пропсы в этом выражении. Пропсы должны быть в выражении должны быть в формате
/// self.{name} и будут также сохоанены в "self.{name}" формате
pub struct PropsFinder<'a> {
    /// Список всех пропсов компонента (имя, тип, id)
    pub props: &'a Vec<(String, String, usize)>,

    /// Найденные пропсы (путь, id)
    pub found: &'a mut Vec<(String, usize)>,
}

impl<'ast> Visit<'ast> for PropsFinder<'_> {
    fn visit_expr_field(&mut self, i: &'ast ExprField) {
        // Так как пропчы это поля структуры любое выражение без self. уже не может быть
        // пропсом компонента
        if let Expr::Path(path_expr) = &*i.base {
            if path_expr.path.is_ident("self") {
                let field_name = i.member.to_token_stream().to_string();

                if let Some((_, _, id)) = self.props.iter().find(|(name, _, _)| name == &field_name) {
                    let full_path = format!("self.{}", field_name);
                    if !self.found.iter().any(|(found_path, _)| found_path == &full_path) {
                        self.found.push((full_path, *id));
                    }
                }
            }
        }

        visit::visit_expr_field(self, i);
    }
    
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if let Expr::Path(path_expr) = &*i.receiver {
            if path_expr.path.is_ident("self") {
                let method_name = i.method.to_string();
                
                if let Some((_, _, id)) = self.props.iter().find(|(name, _, _)| name == &method_name) {
                    let full_path = format!("self.{}", method_name);
                    if !self.found.iter().any(|(found_path, _)| found_path == &full_path) {
                        self.found.push((full_path, *id));
                    }
                }
            }
        }
        
        visit::visit_expr_method_call(self, i);
    }
}

impl<'a> PropsFinder<'a> {
    /// Создает новый экземпляр PropsFinder
    pub fn new(props: &'a Vec<(String, String, usize)>, found: &'a mut Vec<(String, usize)>) -> Self {
        Self {
            props,
            found,
        }
    }
    
    /// Анализирует выражение и возвращает найденные пропсы
    pub fn analyze(&mut self, expr: &Expr) -> &Vec<(String, usize)> {
        self.visit_expr(expr);
        self.found
    }
}

/// Упрощенная функция для быстрого поиска пропсов в выражении
pub fn find_props_in_expr(
    expr: &Expr,
    props: &Vec<(String, String, usize)>,
) -> Vec<(String, usize)> {
    let mut found = Vec::new();
    let mut finder = PropsFinder::new(props, &mut found);

    finder.analyze(expr);
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;
    
    fn create_test_props() -> Vec<(String, String, usize)> {
        vec![
            ("counter".to_string(), "i32".to_string(), 1),
            ("name".to_string(), "String".to_string(), 2),
            ("items".to_string(), "Vec<i32>".to_string(), 3),
            ("is_active".to_string(), "bool".to_string(), 4),
        ]
    }
    
    #[test]
    fn test_find_simple_prop() {
        let props = create_test_props();
        let expr: Expr = parse_quote! {
            self.counter
        };
        
        let result = find_props_in_expr(&expr, &props);
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], ("self.counter".to_string(), 1));
    }
    
    #[test]
    fn test_find_multiple_props() {
        let props = create_test_props();
        let expr: Expr = parse_quote! {
            self.counter + self.name.len()
        };
        
        let result = find_props_in_expr(&expr, &props);
        
        assert_eq!(result.len(), 2);
        assert!(result.contains(&("self.counter".to_string(), 1)));
        assert!(result.contains(&("self.name".to_string(), 2)));
    }
    
    #[test]
    fn test_find_prop_in_method_call() {
        let props = create_test_props();
        let expr: Expr = parse_quote! {
            self.items.push(10)
        };
        
        let result = find_props_in_expr(&expr, &props);
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], ("self.items".to_string(), 3));
    }
    
    #[test]
    fn test_find_prop_in_nested_expression() {
        let props = create_test_props();
        let expr: Expr = parse_quote! {
            (self.counter * 2) + self.is_active
        };
        
        let result = find_props_in_expr(&expr, &props);
        
        assert_eq!(result.len(), 2);
        assert!(result.contains(&("self.counter".to_string(), 1)));
        assert!(result.contains(&("self.is_active".to_string(), 4)));
    }
    
    #[test]
    fn test_ignore_non_self_fields() {
        let props = create_test_props();
        let expr: Expr = parse_quote! {
            some_var.counter + self.name
        };
        
        let result = find_props_in_expr(&expr, &props);
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], ("self.name".to_string(), 2));
    }
    
    #[test]
    fn test_no_duplicates() {
        let props = create_test_props();
        let expr: Expr = parse_quote! {
            self.counter + self.counter
        };
        
        let result = find_props_in_expr(&expr, &props);
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], ("self.counter".to_string(), 1));
    }
}
