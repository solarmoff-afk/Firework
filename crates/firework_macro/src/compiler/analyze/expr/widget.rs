// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use std::fmt;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, Ident, Lit, Result, Token, punctuated::Punctuated};

/// Струкутра для хранения информации о инициализация поля внутри декларативного виджета,
/// пример:
///
/// widget!(
///  // WidgetProperty
///  field1: 10, // Имя/name (field1) и выражение/expr (10)
/// );
pub struct WidgetProperty {
    // Левая часть, имя поля
    pub name: Ident,

    // Правая часть, выражение которое задаётся для этого поля
    pub value: Expr,

    pub attrs: Vec<WidgetPropertyAttribute>,
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

        // Центральная часть, пропускается так как не нужна. Двоеточие которое
        // разделяет левую и правую часть (a: b)
        let _: Token![:] = input.parse()?;

        // Правая часть, выражение которое может быть замыканием с телом в фигурных скобках
        let value: Expr = if input.peek(syn::token::Brace) {
            // Возможно замыкание
            let block: syn::ExprBlock = input.parse()?;
            syn::Expr::Block(block)
        } else {
            input.parse()?
        };

        Ok(WidgetProperty { attrs, name, value })
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
        Ok(WidgetArgs {
            properties: input.parse_terminated(WidgetProperty::parse, Token![,])?,
        })
    }
}

/// Эта функция нужна чтобы определить явлется ли имя макроса декларативным виджетом
/// firework. Виджеты это строительные блоки пользовательского интерфейса которые
/// разворачиваются в примитивы. Синтаксис виджета это widget_name!(field1: 10);
pub fn is_widget(name: &str) -> bool {
    name == "rect"      ||
    name == "text"      ||
    name == "button"    ||
    name == "app_bar"   ||

    // Функциональные виджеты, они не имеют набора рендер примитивов (скина) и
    // нужны для выполнения логики с синтаксисом DSL виджет
    name == "component" ||
    name == "layout"
}

/// Эта функция определяет явлется ли макрос лайаутом, лайаут это контейнер который
/// хранит виджеты и настраивается через функциональный виджет layout!, тело
/// лайаута задаётся в фигурных скобках
///
/// Пример:
/// vertical! {
///    // Код
/// };
pub fn is_layout(name: &str) -> bool {
    name == "vertical" || name == "horizontal" || name == "stack" || name == "absoulute"
}

/// Является ли это функциональным виджетом
pub fn is_functional_widget(name: &str) -> bool {
    name == "layout" || name == "component"
}

/// Принимает имя виджета, возвращает тип скина который использует этот виджет. Это структура
/// которую нужно положить в структуру экрана или компонента которая содержит хэндлы рендер
/// примитивов
pub fn map_skin(widget_name: &str) -> Option<String> {
    match widget_name {
        "rect" => Some("firework_ui::skins::DefaultRectSkin".to_string()),

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
