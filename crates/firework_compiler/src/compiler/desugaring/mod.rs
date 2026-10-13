// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use syn::parse::Parser;

use quote::quote;
use syn::visit_mut::VisitMut;
use syn::{
    Expr, ExprBlock, ExprClosure, File, ItemMacro, Macro, parse_quote_spanned, spanned::Spanned,
};

use crate::compiler::analyze::expr::widget::WidgetArgs;
use crate::compiler::common::widget_kind::{is_layout, is_widget};

pub struct DesugarVisitor;

impl VisitMut for DesugarVisitor {
    /// Разворачивает короткое замыкание (без блока) в замыкание с блоком внутри. Это
    /// нужно для того, чтобы правильно разворачивать стейтементы let mut cl = || a += 1;
    /// где a это спарк. Сворачивание в блок позволяет кодогенератору правильно сгенерировать
    /// обвязку (в данном случае, изменение битовой маски) для этого выражения, так как
    /// оно теперь в блоке (let mut cl = || a += 1;)
    fn visit_expr_closure_mut(&mut self, i: &mut ExprClosure) {
        syn::visit_mut::visit_expr_closure_mut(self, i);

        if !matches!(*i.body, Expr::Block(_)) {
            let body_expr = &*i.body;
            let span = body_expr.span();

            let new_block: ExprBlock = parse_quote_spanned!(span=>
                { #body_expr }
            );

            *i.body = Expr::Block(new_block);
        }
    }

    fn visit_item_macro_mut(&mut self, i: &mut ItemMacro) {
        if let Some(segment) = i.mac.path.segments.last() {
            let name = segment.ident.to_string();

            if name == "component" || name == "shared" {
                if let Ok(mut file) = syn::parse2::<syn::File>(i.mac.tokens.clone()) {
                    self.visit_file_mut(&mut file);

                    let items = file.items;
                    i.mac.tokens = quote!(#(#items)*);

                    return;
                }
            }
        }

        syn::visit_mut::visit_item_macro_mut(self, i);
    }

    fn visit_macro_mut(&mut self, i: &mut Macro) {
        if let Some(segment) = i.path.segments.last() {
            let name = segment.ident.to_string();

            if is_layout(&name) {
                let tokens = &i.tokens;
                let tokens_with_braces = quote!({ #tokens });

                if let Ok(mut block) = syn::parse2::<syn::Block>(tokens_with_braces) {
                    self.visit_block_mut(&mut block);
                    let stmts = block.stmts;
                    i.tokens = quote!(#(#stmts)*);
                }
            } else if is_widget(&name) {
                if let Ok(args) = syn::parse2::<WidgetArgs>(i.tokens.clone()) {
                    let mut new_tokens = proc_macro2::TokenStream::new();

                    for (idx, mut prop) in args.properties.into_iter().enumerate() {
                        if idx > 0 {
                            new_tokens.extend(quote!(,));
                        }

                        self.visit_expr_mut(&mut prop.value);

                        for attr in &prop.attrs {
                            let attr_name = &attr.name;
                            if let Some(attr_args) = &attr.args {
                                let args_str: Vec<String> =
                                    attr_args.iter().map(|a| a.to_string()).collect();
                                let args_joined = args_str.join(", ");
                                let attr_str = format!("#[{}({})]", attr_name, args_joined);
                                if let Ok(ts) = attr_str.parse::<proc_macro2::TokenStream>() {
                                    new_tokens.extend(ts);
                                }
                            } else {
                                let attr_str = format!("#[{}]", attr_name);
                                if let Ok(ts) = attr_str.parse::<proc_macro2::TokenStream>() {
                                    new_tokens.extend(ts);
                                }
                            }
                        }

                        let prop_name = &prop.name;
                        let prop_value = &prop.value;
                        new_tokens.extend(quote!(#prop_name: #prop_value));
                    }

                    i.tokens = new_tokens;
                }
            } else if name == "effect" {
                if let Ok(mut exprs) =
                    syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated
                        .parse2(i.tokens.clone())
                {
                    for expr in exprs.iter_mut() {
                        self.visit_expr_mut(expr);
                    }
                    i.tokens = quote!(#exprs);
                }
            }
        }
        syn::visit_mut::visit_macro_mut(self, i);
    }
}

pub fn normalize_ast(file: &mut File) {
    let mut visitor = DesugarVisitor;
    visitor.visit_file_mut(file);
}
