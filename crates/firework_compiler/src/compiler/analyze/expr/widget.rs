// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use std::fmt;
use syn::parse::{Parse, ParseStream, discouraged::Speculative};
use syn::{Expr, Ident, Lit, Result, Token, punctuated::Punctuated};

fn extract_index(ident: &Ident) -> Option<usize> {
    let name_str = ident.to_string();
    let digits: String = name_str
        .chars()
        .rev()
        .take_while(|c| c.is_ascii_digit())
        .collect();

    if digits.is_empty() {
        None
    } else {
        digits.chars().rev().collect::<String>().parse().ok()
    }
}

fn mangle_component(name: &Ident) -> String {
    let name_str = name.to_string();
    let digits: String = name_str
        .chars()
        .rev()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    let digits: String = digits.chars().rev().collect();
    let base_name = if digits.is_empty() {
        name_str.as_str()
    } else {
        &name_str[..name_str.len() - digits.len()]
    };

    if digits.is_empty() {
        format!("fwci_{}", base_name)
    } else {
        format!("fwci{}_{}", digits, base_name)
    }
}

fn flatten_into(
    mut prop: WidgetProperty,
    parent_path: String,
    out: &mut Punctuated<WidgetProperty, Token![,]>,
) {
    if let Some(nested) = prop.nested.take() {
        let current_comp = mangle_component(&prop.name);
        let new_path = if parent_path.is_empty() {
            current_comp
        } else {
            format!("{}__{}", parent_path, current_comp)
        };

        // Рекурсивно сглаживаем детей
        for child in nested {
            flatten_into(child, new_path.clone(), out);
        }
    } else {
        if parent_path.is_empty() {
            // Обычное свойство верхнего уровня
            out.push(prop);
        } else {
            // Вложенное свойство (leaf) генерируем полное имя
            let current_comp = mangle_component(&prop.name);
            let final_path = format!("_fwcpar_{}__{}", parent_path, current_comp);
            let new_ident = Ident::new(&final_path, prop.name.span());

            prop.name = new_ident;
            out.push(prop);
        }
    }
}

/// Струкутра для хранения информации о инициализация поля внутри декларативного виджета,
/// пример:
///
/// widget!(
///  // WidgetProperty
///  field1: 10, // Имя/name (field1) и выражение/expr (10)
///  field2, // Синтаксический сахар
/// );
pub struct WidgetProperty {
    // Левая часть, имя поля
    pub name: Ident,

    // Правая часть, выражение которое задаётся для этого поля
    pub value: Expr,

    pub attrs: Vec<WidgetPropertyAttribute>,

    #[allow(dead_code)]
    pub index: Option<usize>,

    #[allow(dead_code)]
    pub is_relative: bool,

    pub nested: Option<Vec<WidgetProperty>>,
}

impl Parse for WidgetProperty {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attrs = Vec::new();
        while input.peek(Token![#]) {
            let attr: WidgetPropertyAttribute = input.parse()?;
            attrs.push(attr);
        }

        // Левая часть, имя поля куда задаётся значение
        let name: Ident = input.parse()?;
        let index = extract_index(&name);

        // Синтаксический сахар, двоеточие необязательно если поле это простой идент
        if input.peek(Token![:]) {
            // Центральная часть, пропускается так как не нужна. Двоеточие которое
            // разделяет левую и правую часть (a: b)
            let _: Token![:] = input.parse()?;

            let mut value: Expr;
            let mut nested = None;
            let mut parsed_as_nested = false;

            // Правая часть, выражение которое может быть замыканием с телом в фигурных скобках
            if input.peek(syn::token::Brace) {
                let fork = input.fork();
                let content;
                let _brace_token = syn::braced!(content in fork);

                // Пытаемся распарсить содержимое как свойства (для вложенных блоков)
                if let Ok(args) = content.parse_terminated(WidgetProperty::parse, Token![,]) {
                    if content.is_empty() {
                        input.advance_to(&fork);
                        nested = Some(args.into_iter().collect());
                        parsed_as_nested = true;
                    }
                }
            }

            if parsed_as_nested {
                value = syn::parse_quote!(());
            } else if input.peek(syn::token::Brace) {
                // Возможно замыкание
                let block: syn::ExprBlock = input.parse()?;
                value = syn::Expr::Block(block);
            } else {
                value = input.parse()?;
            }

            let mut is_relative = false;
            if let Expr::Macro(expr_mac) = &value {
                if expr_mac.mac.path.is_ident("relative") {
                    is_relative = true;
                    value = syn::parse2(expr_mac.mac.tokens.clone())?;
                }
            }

            Ok(WidgetProperty {
                attrs,
                name,
                value,
                index,
                is_relative,
                nested,
            })
        } else {
            let value = Expr::Path(syn::ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: name.clone().into(),
            });

            Ok(WidgetProperty {
                attrs,
                name,
                value,
                index,
                is_relative: false,
                nested: None,
            })
        }
    }
}

impl WidgetProperty {
    pub fn get_attribute(&self, name: &str) -> Option<&WidgetPropertyAttribute> {
        self.attrs.iter().find(|attr| attr.name == name)
    }
}

/// Структура для хранения списка полей виджета которые разделяются через запятую
/// пример:
/// widget!(
///  name: 1,  // WidgetProperty
///  name2: 2, // WidgetProperty
///  name3: 3, // WidgetProperty
/// )
///
/// Все WidgetProperty виджета хранятся здесь
pub struct WidgetArgs {
    // Запятая стандартный способ в расте разграничить поля
    pub properties: Punctuated<WidgetProperty, Token![,]>,
}

impl Parse for WidgetArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let parsed = input.parse_terminated(WidgetProperty::parse, Token![,])?;

        // Сглаживание дерева: разворачиваем вложенные блоки
        let mut properties = Punctuated::new();
        for prop in parsed {
            flatten_into(prop, String::new(), &mut properties);
        }

        Ok(WidgetArgs { properties })
    }
}

/// Принимает имя виджета, возвращает тип скина который использует этот виджет. Это структура
/// которую нужно положить в структуру экрана или компонента которая содержит хэндлы рендер
/// примитивов
pub fn map_skin(widget_name: &str) -> Option<String> {
    match widget_name {
        "rect" => Some("firework_ui::DefaultRectSkin".to_string()),
        "text" => Some("firework_ui::DefaultTextSkin".to_string()),
        "timer" => Some("firework_ui::Timer".to_string()),

        // Не имеет скина так как явлется функциональным виджетом
        _ => None,
    }
}

/// Атрибут для поля виджета
#[derive(Debug, Clone)]
pub struct WidgetPropertyAttribute {
    pub name: Ident,
    pub args: Option<Vec<WidgetAttributeArg>>,
}

#[derive(Debug, Clone)]
pub enum WidgetAttributeArg {
    Lit(Lit),
    Ident(Ident),
    Tuple(Vec<WidgetAttributeArg>),
}

impl Parse for WidgetAttributeArg {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            let args = Punctuated::<WidgetAttributeArg, Token![,]>::parse_terminated(&content)?;
            Ok(WidgetAttributeArg::Tuple(args.into_iter().collect()))
        } else if input.peek(syn::Lit) {
            Ok(WidgetAttributeArg::Lit(input.parse()?))
        } else {
            Ok(WidgetAttributeArg::Ident(input.parse()?))
        }
    }
}

impl Parse for WidgetPropertyAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let _: Token![#] = input.parse()?;
        let content;
        syn::bracketed!(content in input);

        let name: Ident = content.parse()?;
        let args = if content.peek(syn::token::Paren) {
            let args_content;
            syn::parenthesized!(args_content in content);
            let args =
                Punctuated::<WidgetAttributeArg, Token![,]>::parse_terminated(&args_content)?;
            Some(args.into_iter().collect())
        } else {
            None
        };

        Ok(WidgetPropertyAttribute { name, args })
    }
}

impl fmt::Display for WidgetAttributeArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WidgetAttributeArg::Lit(lit) => {
                write!(f, "{}", quote::quote!(#lit))
            }

            WidgetAttributeArg::Ident(ident) => {
                write!(f, "{}", ident)
            }

            WidgetAttributeArg::Tuple(args) => {
                write!(f, "(")?;

                for (position, argument) in args.iter().enumerate() {
                    if position > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{}", argument)?;
                }

                write!(f, ")")
            }
        }
    }
}
