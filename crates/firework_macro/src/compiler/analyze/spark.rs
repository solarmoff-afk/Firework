// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use syn::*;
use syn::parse::{Parse, ParseStream};
use syn::visit::Visit;
use quote::ToTokens;
use proc_macro2::{TokenStream, Span};

use super::Scope;

/// Валидатор реактивных инициализаций (спарков). Собирает все инициализации спарков в
/// выражении и заполняет spark_count (их количество) и spark_tokens, это токены внутри
/// макроса spark!(...)
pub struct SparkValidator {
    // Сколько вызовов spark! найдено в текущем выражении (expr). Это важно чтобы
    // избежать выражений spark!() + spark!() которые нельзя нормально разместить
    // в статическом графе зависимостей
    pub spark_count: usize,

    // Хранит в себе токены последнего спарка который был найден
    pub spark_tokens: Option<TokenStream>,

    // Выражение внутри spark!()
    pub spark_expr: Option<Expr>,
}

impl<'ast> Visit<'ast> for SparkValidator {
    fn visit_expr_macro(&mut self, i: &'ast ExprMacro) {
        // Проверка что вызов действительно spark!
        if i.mac.path.is_ident("spark") {
            // Добавление единицы к значению всех вызовов spark в выражении 
            self.spark_tokens = Some(i.mac.tokens.clone());

            // Попытка парсинга как выражение
            if let Ok(expr) = syn::parse2::<Expr>(i.mac.tokens.clone()) {
                self.spark_expr = Some(expr);
            }

            self.spark_count += 1;
        }

        // Возможно макрос внутри макроса, тут запуск парсинга такого выражения
        // работает потому-что даже если внутри макроса spark!(...) будет макрос
        // spark!(...) то добавится self.spark_count и он уже не будет равен 0 
        // или 1, а значит будет ошибка
        visit::visit_expr_macro(self, i);
    }
}

pub struct SparkFinder<'a> {
    pub scope: &'a Scope,
    pub found: &'a mut Vec<String>,
}

impl<'ast> Visit<'ast> for SparkFinder<'_> {
    fn visit_expr_path(&mut self, i: &'ast ExprPath) {
        let var_name = i.path.to_token_stream().to_string();

        if let Some(var) = self.scope.variables.get(&var_name) {
            if var.is_spark {
                if !self.found.contains(&var_name) {
                    self.found.push(var_name);
                }
            }
        }
    }
}

pub struct SparkFinderWithId<'a> {
    pub scope: &'a Scope,
    pub found: &'a mut Vec<(String, usize)>,
}

impl<'ast> Visit<'ast> for SparkFinderWithId<'_> {
    fn visit_expr_path(&mut self, i: &'ast ExprPath) {
        let var_name = i.path.to_token_stream().to_string();

        if let Some(var) = self.scope.variables.get(&var_name) {
            if var.is_spark {
                let id = var.spark_id;
                
                if !self.found.contains(&(var_name.clone(), id)) {
                    self.found.push((var_name, id));
                }
            }
        }
    }
}

/// Эта функция позволяет узнать корень выражения чтобы потом понять явлется ли это
/// работой со спарком или нет в выражениях с полями на более высоком уровне анализатора
/// Пример:
///  spark1.field.subfield = 5;
///
///   spark1 - Корень
///   field1 - Поле
///   subfield - Поле
///
/// Также поддерживается обнаружение корня при индексации, работе с ссылкой и так далее
/// Зная это выражение нам нужно получить имя корня и вернуть его
pub fn get_root_variable_name(expr: &Expr) -> Option<String> {
    match expr {
        // Прямое использование спарка
        Expr::Path(path_expr) => {
            Some(path_expr.to_token_stream().to_string())
        },

        // Через поле, например spark1.field. В таком случае нужно запустить эту
        // функцию для базы (левого соседа) выражения и если там будет например
        // spark1.field.subfield то рекурсия выполнится для базы subfield,
        // дальше функция найдёт field и поймёт что это тоже поле и на 2 вызов
        // рекурсии найдёт корень (path), в этом примере это spark1
        Expr::Field(field_expr) => {
            get_root_variable_name(&field_expr.base)
        },

        // Индексация (например в векторах или массивах) spark1[10]
        Expr::Index(index_expr) => {
            get_root_variable_name(&index_expr.expr)
        },

        Expr::Paren(paren) => get_root_variable_name(&paren.expr),

        // Ссылки
        Expr::Reference(reference) => get_root_variable_name(&reference.expr),

        // Корень не найден
        _ => None
    }
}

/// Структура для храненеия глобального спарка для shared! {} 
#[derive(Debug)]
pub struct GlobalState {
    pub name: Ident,
    pub spark_type: Type,
    pub init: Expr,
    pub span: Span,
}

/// Парсер глобального состояния в state! {} для shared юнитов
impl Parse for GlobalState {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let span = input.span();

        let name: Ident = input.parse()?;
        let _: Token![:] = input.parse()?;
        
        let spark_type: Type = input.parse()?;
        let _: Token![=] = input.parse()?;
        
        let init: Expr = input.parse()?;
        
        Ok(GlobalState {
            name,
            spark_type,
            init,
            span,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::analyze::lifetime_manager::Variable; 
    use syn::parse_quote;

    #[test]
    fn test_spark_validator_counts_spark_macros() {
        let mut validator = SparkValidator {
            spark_count: 0,
            spark_tokens: None,
            spark_expr: None,
        };

        let expr: Expr = parse_quote! {
            spark!(5 + 3)
        };

        validator.visit_expr(&expr);

        assert_eq!(validator.spark_count, 1);
        assert!(validator.spark_tokens.is_some());
        assert!(validator.spark_expr.is_some());
    }

    #[test]
    fn test_spark_validator_counts_multiple_sparks() {
        let mut validator = SparkValidator {
            spark_count: 0,
            spark_tokens: None,
            spark_expr: None,
        };

        let expr: Expr = parse_quote! {
            spark!(10) + spark!(20)
        };

        validator.visit_expr(&expr);

        assert_eq!(validator.spark_count, 2);
    }

    #[test]
    fn test_spark_validator_ignores_other_macros() {
        let mut validator = SparkValidator {
            spark_count: 0,
            spark_tokens: None,
            spark_expr: None,
        };

        let expr: Expr = parse_quote! {
            println!("hello") + spark!(42)
        };

        validator.visit_expr(&expr);

        assert_eq!(validator.spark_count, 1);
    }

    #[test]
    fn test_spark_finder_detects_spark_variables() {
        let mut scope = Scope::new();
        let spark_var = Variable {
            variable_type: "i32".to_string(),
            is_spark: true,
            is_mut: false,
            spark_id: 1,
        };
        scope.variables.insert("my_spark".to_string(), spark_var);

        let mut found = Vec::new();
        let mut finder = SparkFinder {
            scope: &scope,
            found: &mut found,
        };

        let expr: Expr = parse_quote! {
            my_spark + 10
        };

        finder.visit_expr(&expr);

        assert_eq!(found, vec!["my_spark"]);
    }

    #[test]
    fn test_spark_finder_with_id_detects_spark_with_id() {
        let mut scope = Scope::new();
        let spark_var = Variable {
            variable_type: "String".to_string(),
            is_spark: true,
            is_mut: true,
            spark_id: 99,
        };
        scope.variables.insert("reactive_var".to_string(), spark_var);

        let mut found = Vec::new();
        let mut finder = SparkFinderWithId {
            scope: &scope,
            found: &mut found,
        };

        let expr: Expr = parse_quote! {
            reactive_var.field.subfield
        };

        finder.visit_expr(&expr);

        assert_eq!(found, vec![("reactive_var".to_string(), 99)]);
    }
}
