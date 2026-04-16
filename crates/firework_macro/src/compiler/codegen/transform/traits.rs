// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use proc_macro2::TokenStream;
use syn::parse_str;
use syn::*;

/// Конвертация вектора строк в TokenStream
pub(crate) trait ToTokenStreams {
    fn to_token_streams(self) -> syn::Result<Vec<TokenStream>>;
}

impl ToTokenStreams for Vec<String> {
    fn to_token_streams(self) -> syn::Result<Vec<TokenStream>> {
        self.into_iter()
            .map(|s| parse_str(&s))
            .collect()
    }
}

/// Конвертация строки (String или str) в стейтемент syn
pub(crate) trait ToStmt {
    fn to_stmt(&self) -> syn::Result<Stmt>;
}

impl ToStmt for str {
    fn to_stmt(&self) -> syn::Result<Stmt> {
        parse_str(self)
    }
}

impl ToStmt for String {
    fn to_stmt(&self) -> syn::Result<Stmt> {
        parse_str(self)
    }
}

/// Конвертация строки в выражение syn
pub(crate) trait ToExpr {
    fn to_expr(&self) -> syn::Result<syn::Expr>;
}

impl ToExpr for str {
    fn to_expr(&self) -> syn::Result<syn::Expr> {
        parse_str(self)
    }
}

impl ToExpr for String {
    fn to_expr(&self) -> syn::Result<syn::Expr> {
        parse_str(self)
    }
}
