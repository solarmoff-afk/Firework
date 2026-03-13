// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use quote::ToTokens;

use crate::compiler::analyze::prepare::CompilerContext;
use crate::compiler::analyze::statement::parse_stmts;
use crate::compiler::analyze::pattern::parse_pat;
use crate::compiler::analyze::expr::parse_expr;

/// Парсит предметы. Предмет это верхушка пищевой цепи в Rust, это модули, функции,
/// трейты, структуры и так далее
pub fn parse_items(item: syn::Item, context: &mut CompilerContext) -> proc_macro2::TokenStream {
    let mut output = proc_macro2::TokenStream::new();
    
    match &item {
        // TEMP: Заглушка чтобы компилятор не жаловался пока макрос в разработке
        syn::Item::Fn(item_fn) => {
            let fn_name = item_fn.sig.ident.to_string();
            context.log("fn_found", &format!("function: {}", fn_name));

            // Для констант и статичных переменных
            let saved_metadata = context.metadata.clone();

            // Если context.depth это ноль то мы находимся в корне ui! блока, а значит
            // заходим в функцию экрана. Архитектура фреймворка разрешает делать несколько
            // экранов (функций) в одном ui! блоке поэтому для каждого экрана должны
            // быть чистые метаданные, поэтому очищаем их
            context.metadata.clear();

            // Нужно учитывать аргументы функции как переменные тоже
            for input in &item_fn.sig.inputs {
                if let syn::FnArg::Typed(pat_type) = input {
                    let arg_type = pat_type.ty.to_token_stream().to_string();

                    context.is_special_var = true;
                        parse_pat(*pat_type.pat.clone(), Some(arg_type), context);
                    context.is_special_var = false;
                }
            }
            
            context.depth += 1;
                parse_stmts(item_fn.block.stmts.clone(), context);
            context.depth -= 1;
            
            context.log("FN_STUB", &format!("Generating stub for: {}", fn_name));
            
            let mut fn_stub = item_fn.clone();
            fn_stub.block = syn::parse2(quote::quote! {
                {}
            }).expect("Failed to parse item");

            // Возврат метаданных до входа в эту область видимости
            context.metadata = saved_metadata;

            output.extend(quote::quote! {
                #fn_stub
            });
        },
        
        syn::Item::Struct(item_struct) => {
            let struct_name = item_struct.ident.to_string();
            context.log("STRUCT_PASSTHROUGH", &format!("Struct: {}", struct_name));

            output.extend(quote::quote! {
                #item
            });
        },
        
        syn::Item::Enum(item_enum) => {
            let enum_name = item_enum.ident.to_string();
            context.log("ENUM_PASSTHROUGH", &format!("Enum: {}", enum_name));
            
            output.extend(quote::quote! {
                #item
            });
        },
        
        syn::Item::Type(item_type) => {
            let type_name = item_type.ident.to_string();
            context.log("TYPE_PASSTHROUGH", &format!("Type alias: {}", type_name));
            
            output.extend(quote::quote! {
                #item
            });
        },
        
        syn::Item::Const(item_const) => {
            let const_name = item_const.ident.to_string();
            context.log("CONST", &format!("Constant: {}", const_name));
    
            context.metadata.variables.insert(const_name.clone());
            
            context.is_static = true;
                parse_expr(*item_const.expr.clone(), context);
            context.is_static = false;
    
            output.extend(quote::quote! {
                #item
            });
        },

        syn::Item::Static(item_static) => {
            let static_name = item_static.ident.to_string();
            context.log("STATIC", &format!("Static {}", static_name));

            context.metadata.variables.insert(static_name.clone());
            
            context.is_static = true;
                parse_expr(*item_static.expr.clone(), context);
            context.is_static = false;
            
            output.extend(quote::quote! {
                #item
            });
        },
        
        syn::Item::Trait(item_trait) => {
            let trait_name = item_trait.ident.to_string();
            context.log("TRAIT_PASSTHROUGH", &format!("Trait: {}", trait_name));
            output.extend(quote::quote! {
                #item
            });
        },
        
        syn::Item::Impl(_item_impl) => {
            context.log("IMPL_PASSTHROUGH", "Implementation block");
            output.extend(quote::quote! {
                #item
            });
        },
        
        syn::Item::Mod(item_mod) => {
            let mod_name = item_mod.ident.to_string();
            context.log("MOD_PASSTHROUGH", &format!("Module: {} (keeping original)", mod_name));
            
            output.extend(quote::quote! {
                #item_mod
            });
        },

        syn::Item::Use(_item_use) => {
            context.log("USE_PASSTHROUGH", "Use statement");
            
            output.extend(quote::quote! {
                #item
            });
        },
        
        syn::Item::ExternCrate(item_extern) => {
            context.log("EXTERN_CRATE_PASSTHROUGH", &format!("Extern crate: {}", item_extern.ident));
            output.extend(quote::quote! {
                #item
            });
        },
        
        syn::Item::ForeignMod(_item_foreign) => {
            context.log("FOREIGN_MOD_PASSTHROUGH", "Foreign module");
            
            output.extend(quote::quote! {
                #item
            });
        },
        
        syn::Item::Macro(item_macro) => {
            if let Some(last_segment) = item_macro.mac.path.segments.last() {
                let macro_name = &last_segment.ident;
                context.log("MACRO_ITEM_PASSTHROUGH", &format!("Macro: {}!", macro_name));
            }

            output.extend(quote::quote! {
                #item
            });
        },
        
        syn::Item::Verbatim(tokens) => {
            context.log("VERBATIM_PASSTHROUGH", "Raw tokens");
            output.extend(tokens.clone());
        },
        
        _ => {
            context.log("UNKNOWN_ITEM", "Unknown item type");
            
            output.extend(quote::quote! {
                #item
            });
        }
    };
    
    output
}

