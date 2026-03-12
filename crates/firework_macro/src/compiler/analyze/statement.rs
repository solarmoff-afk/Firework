// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use syn::Stmt;

use crate::compiler::widgets::is_widget_macro;
use crate::compiler::codegen::actions::FireworkAction;
use crate::compiler::analyze::prepare::CompilerContext;
use crate::compiler::analyze::pattern::parse_local;
use crate::compiler::analyze::item::parse_items;
use crate::compiler::analyze::expr::parse_expr;


/// Парсит statement, это конкретная команда. Можно упростить и сказать что это
/// строки, но это не совсем так
pub fn parse_stmts(statements: Vec<Stmt>, context: &mut CompilerContext) {
    for statement in statements {
        println!("STATEMENT:");
        println!("{:#?}", statement);
        
        context.last_statement.action = FireworkAction::DefaultCode;

        match statement {
            Stmt::Local(local) => {
                parse_local(local, context);
            },
            
            Stmt::Item(item) => {
                parse_items(item, context);
            },

            Stmt::Expr(expr, _semi) => {
                parse_expr(expr, context);
            },

            Stmt::Macro(mac) => {
                if is_widget_macro(&mac.mac.path) {
                    let inner_tokens = &mac.mac.tokens;

                    let _parser = |input: syn::parse::ParseStream| -> syn::Result<Vec<Stmt>> {
                        let mut stmts = Vec::new();

                        while !input.is_empty() {
                            stmts.push(input.parse::<syn::Stmt>()?);
                        }

                        Ok(stmts)
                    };

                    let inner_stmts: Vec<Stmt> = syn::parse2(quote::quote! {
                        {
                            #inner_tokens
                        }
                    }).and_then(|block: syn::Block| Ok(block.stmts))
                        .expect("Syntax error in macro");
                    
                    parse_stmts(inner_stmts, context);
                } else {
                    if let Some(last_segment) = mac.mac.path.segments.last() {
                        let _macro_name = &last_segment.ident;
                        
                        // Это обычный макрос, нужен инлайн
                    }
                }
            },
        };

        println!("Statement index: {}", context.statement_index);
        context.statement_index += 1;
        context.last_statement.index += 1;

        context.statements.push(context.last_statement.clone());
    }
}

