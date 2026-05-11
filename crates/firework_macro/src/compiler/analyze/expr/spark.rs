// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

#![allow(dead_code)]

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::parse::Parser;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::*;

use super::super::Scope;

use crate::compiler::error::{
    SPARK_ASYNC_CLOSURE_ERROR, SPARK_ASYNC_KEYWORD_ERROR, SPARK_ASYNC_MOVE_ERROR,
    SPARK_ASYNC_SYNTAX_ERROR,
};

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

    // Спарк поддерживает синтаксис spark!(v, async), здесь хранятся аргументы замыкания
    // кроме первого аргумента (там всегда контроллер)
    pub spark_async_closure: Option<(Vec<(String, String)>, TokenStream)>,

    // Ошибки парсинга
    pub spark_parse_error: Option<(String, Span)>,
}

impl SparkValidator {
    fn parse_async_closure(
        &mut self,
        tokens: TokenStream,
    ) -> std::result::Result<(Vec<(String, String)>, TokenStream), String> {
        use syn::parse::Parser;

        let args = syn::punctuated::Punctuated::<Expr, Token![,]>::parse_terminated
            .parse2(tokens)
            .map_err(|_| SPARK_ASYNC_SYNTAX_ERROR.to_string())?;

        if args.len() != 2 {
            return Err(SPARK_ASYNC_SYNTAX_ERROR.to_string());
        }

        let closure = match &args[1] {
            Expr::Closure(c) => c,
            _ => return Err(SPARK_ASYNC_CLOSURE_ERROR.to_string()),
        };

        if closure.asyncness.is_none() {
            return Err(SPARK_ASYNC_KEYWORD_ERROR.to_string());
        }

        match closure.capture {
            Some(_) => {}
            None => return Err(SPARK_ASYNC_MOVE_ERROR.to_string()),
        }

        // Пропускаем первый аргумент
        let mut additional_args = Vec::new();
        for arg in closure.inputs.iter().skip(1) {
            if let Pat::Type(pat_type) = arg {
                let name = pat_type.pat.to_token_stream().to_string();
                let ty = pat_type.ty.to_token_stream().to_string();
                additional_args.push((name, ty));
            }
        }

        Ok((additional_args, closure.body.to_token_stream()))
    }
}

impl<'ast> Visit<'ast> for SparkValidator {
    fn visit_expr_macro(&mut self, i: &'ast ExprMacro) {
        // Проверка что вызов действительно spark!
        if i.mac.path.is_ident("spark") {
            // Добавление единицы к значению всех вызовов spark в выражении
            self.spark_tokens = Some(i.mac.tokens.clone());

            let args = syn::punctuated::Punctuated::<Expr, Token![,]>::parse_terminated
                .parse2(i.mac.tokens.clone());

            match args {
                // Если есть второй аргумент то это spark!(v, async), необходимо распарсить
                // это как асинхронный спарк
                Ok(args) if args.len() == 2 => {
                    self.spark_expr = Some(args[0].clone());
                    self.spark_tokens = Some(args[0].to_token_stream());

                    match self.parse_async_closure(i.mac.tokens.clone()) {
                        Ok((args, body)) => {
                            self.spark_async_closure = Some((args, body));
                        }

                        Err(_) => {
                            self.spark_parse_error =
                                Some((SPARK_ASYNC_SYNTAX_ERROR.to_string(), i.span()));
                        }
                    }
                }

                _ => {
                    // Попытка парсинга как выражение
                    if let Ok(expr) = syn::parse2::<Expr>(i.mac.tokens.clone()) {
                        self.spark_expr = Some(expr);
                    }
                }
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

        if let Some(var) = self.scope.variables.get(&var_name)
            && var.is_spark
            && !self.found.contains(&var_name)
        {
            self.found.push(var_name);
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

        if let Some(var) = self.scope.variables.get(&var_name)
            && var.is_spark
        {
            let id = var.spark_id;

            if !self.found.contains(&(var_name.clone(), id)) {
                self.found.push((var_name, id));
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
        Expr::Path(path_expr) => Some(path_expr.to_token_stream().to_string()),

        // Через поле, например spark1.field. В таком случае нужно запустить эту
        // функцию для базы (левого соседа) выражения и если там будет например
        // spark1.field.subfield то рекурсия выполнится для базы subfield,
        // дальше функция найдёт field и поймёт что это тоже поле и на 2 вызов
        // рекурсии найдёт корень (path), в этом примере это spark1
        Expr::Field(field_expr) => get_root_variable_name(&field_expr.base),

        // Индексация (например в векторах или массивах) spark1[10]
        Expr::Index(index_expr) => get_root_variable_name(&index_expr.expr),

        Expr::Paren(paren) => get_root_variable_name(&paren.expr),

        // Ссылки
        Expr::Reference(reference) => get_root_variable_name(&reference.expr),

        // Deref (*name)
        Expr::Unary(unary_expr) if matches!(unary_expr.op, UnOp::Deref(_)) => {
            get_root_variable_name(&unary_expr.expr)
        }

        // Корень не найден
        _ => None,
    }
}

/// Структура для храненеия глобального спарка для shared! {}
#[derive(Debug)]
pub struct GlobalState {
    pub name: Ident,
    pub spark_type: Type,
    pub init: Expr,
    pub span: Span,
    pub attributes: Vec<String>,
}

/// Парсер глобального состояния в state! {} для shared юнитов
impl Parse for GlobalState {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let attrs = input.call(Attribute::parse_outer)?;

        let attributes: Vec<String> = attrs
            .iter()
            .filter_map(|attr| {
                let path = &attr.path();
                if path.segments.len() == 1 {
                    let ident = &path.segments.first().unwrap().ident;
                    Some(ident.to_string())
                } else {
                    None
                }
            })
            .collect();

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
            attributes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::analyze::lifetime::Variable;
    use syn::parse_quote;

    #[test]
    fn test_spark_validator_counts_spark_macros() {
        let mut validator = SparkValidator {
            spark_count: 0,
            spark_tokens: None,
            spark_expr: None,
            spark_async_closure: None,
            spark_parse_error: None,
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
            spark_async_closure: None,
            spark_parse_error: None,
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
            spark_async_closure: None,
            spark_parse_error: None,
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
            is_spark_ref: None,
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
            is_spark_ref: None,
        };
        scope
            .variables
            .insert("reactive_var".to_string(), spark_var);

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

    #[test]
    fn test_spark_validator_async_closure() {
        let mut validator = SparkValidator {
            spark_count: 0,
            spark_tokens: None,
            spark_expr: None,
            spark_async_closure: None,
            spark_parse_error: None,
        };

        let expr: Expr = parse_quote! {
            spark!(1, async move |bridge, ctx: Context| {
                ctx.process().await
            })
        };

        validator.visit_expr(&expr);

        assert_eq!(validator.spark_count, 1);
        assert!(validator.spark_async_closure.is_some());
        assert!(validator.spark_expr.is_none());
        assert!(validator.spark_parse_error.is_none());

        let (args, body) = validator.spark_async_closure.unwrap();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0].0, "ctx");
        assert_eq!(args[0].1, "Context");
        assert!(body.to_string().contains("process"));
    }

    #[test]
    fn test_spark_validator_async_closure_multiple_args() {
        let mut validator = SparkValidator {
            spark_count: 0,
            spark_tokens: None,
            spark_expr: None,
            spark_async_closure: None,
            spark_parse_error: None,
        };

        let expr: Expr = parse_quote! {
            spark!(1, async move |bridge, ctx: Context, data: String, counter: i32| {
                ctx.process(data, counter).await
            })
        };

        validator.visit_expr(&expr);

        assert_eq!(validator.spark_count, 1);
        assert!(validator.spark_async_closure.is_some());

        let (args, _) = validator.spark_async_closure.unwrap();
        assert_eq!(args.len(), 3);
        assert_eq!(args[0], ("ctx".to_string(), "Context".to_string()));
        assert_eq!(args[1], ("data".to_string(), "String".to_string()));
        assert_eq!(args[2], ("counter".to_string(), "i32".to_string()));
    }

    #[test]
    fn test_spark_validator_async_without_move_error() {
        let mut validator = SparkValidator {
            spark_count: 0,
            spark_tokens: None,
            spark_expr: None,
            spark_async_closure: None,
            spark_parse_error: None,
        };

        let expr: Expr = parse_quote! {
            spark!(1, async |bridge| {})
        };

        validator.visit_expr(&expr);

        assert_eq!(validator.spark_count, 1);
        assert!(validator.spark_parse_error.is_some());
        assert!(validator.spark_async_closure.is_none());
    }

    #[test]
    fn test_spark_validator_async_without_async_error() {
        let mut validator = SparkValidator {
            spark_count: 0,
            spark_tokens: None,
            spark_expr: None,
            spark_async_closure: None,
            spark_parse_error: None,
        };

        let expr: Expr = parse_quote! {
            spark!(1, move |bridge| {})
        };

        validator.visit_expr(&expr);

        assert_eq!(validator.spark_count, 1);
        assert!(validator.spark_parse_error.is_some());
        assert!(validator.spark_async_closure.is_none());
    }

    #[test]
    fn test_spark_validator_async_invalid_syntax() {
        let mut validator = SparkValidator {
            spark_count: 0,
            spark_tokens: None,
            spark_expr: None,
            spark_async_closure: None,
            spark_parse_error: None,
        };

        let expr: Expr = parse_quote! {
            spark!(1, something_else)
        };

        validator.visit_expr(&expr);

        assert_eq!(validator.spark_count, 1);
        assert!(validator.spark_parse_error.is_some());
        assert!(validator.spark_async_closure.is_none());
    }
}
