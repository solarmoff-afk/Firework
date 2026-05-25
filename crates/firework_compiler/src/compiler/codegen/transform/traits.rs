// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use proc_macro2::TokenStream;
use syn::parse_str;

/// Конвертация вектора строк в TokenStream
pub(crate) trait ToTokenStreams {
    fn to_token_streams(self) -> syn::Result<Vec<TokenStream>>;
}

impl ToTokenStreams for Vec<String> {
    fn to_token_streams(self) -> syn::Result<Vec<TokenStream>> {
        self.into_iter().map(|s| parse_str(&s)).collect()
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
