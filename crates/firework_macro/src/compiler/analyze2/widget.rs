// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use syn::parse::{Parse, ParseStream};
use syn::{Ident, Token, Expr, Result, punctuated::Punctuated};

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
}

impl Parse for WidgetProperty {
    fn parse(input: ParseStream) -> Result<Self> {
        // Левая часть, имя поля куда задаётся значение 
        let name: Ident = input.parse()?;
        
        // Центральная часть, пропускается так как не нужна. Двоеточие которое
        // разделяет левую и правую часть (a: b)
        let _: Token![:] = input.parse()?;
        
        // Правая часть, выражение
        let value: Expr = input.parse()?;

        Ok(WidgetProperty {
            name,
            value
        })
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
    name == "vertical"   ||
    name == "horizontal" ||
    name == "stack"      ||
    name == "absoulute"
}
