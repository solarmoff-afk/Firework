// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use quote::ToTokens;

use crate::compiler::analyze::prepare::CompilerContext;
use crate::compiler::analyze::expr::parse_expr;
use crate::compiler::analyze::prepare::VariableDeclaration;
use crate::compile_error_spanned;
use crate::{SPARK_SHADOWING_ERROR};

/// Парсит создание переменной
pub fn parse_local(local: syn::Local, context: &mut CompilerContext) {
    parse_pat(local.pat, None, context);

    if let Some(init) = local.init {
        context.is_right_side = true;
        context.depth += 1;
            parse_expr(*init.expr, context);
            
            // Нужно добавлять переменную только после обработки макроса spark чтобы
            // избежать механизма который бросает FE004, обработка макроса spark как раз
            // находится в одной из веток parse_expr, поэтому делаем insert только после
            // вызова parse_expr
            context.metadata.variables.insert(context.variable_name.clone());
        context.depth -= 1;
        context.is_right_side = false;
    }
}

/// Парсит паттерн, это может быть a, (a, b), [a, b, c] и так далее
pub fn parse_pat(pat: syn::Pat, current_type: Option<String>, context: &mut CompilerContext) {
    match pat {
        syn::Pat::Ident(ident) => {
            let variable = VariableDeclaration {
                name: ident.ident.to_string(),
                ty: current_type.clone(),
                is_mut: ident.mutability.is_some(),
            };

            let variable_type = current_type.unwrap_or("NO TYPE".to_string()); 

            println!(
                "{}Let: is_mut: {}, name: {}, type: {}",
                context.indent(), variable.is_mut, variable.name,
                variable_type,
            );

            context.variable_name = ident.ident.to_string();
            context.variable_type = variable_type;

            if context.metadata.sparks.contains(&context.variable_name) {
                context.compile_errors.push(compile_error_spanned(
                    &ident,
                    SPARK_SHADOWING_ERROR,
                ));
            }

            if context.is_special_var {
                context.metadata.variables.insert(variable.name.clone());
            }

            context.active_targets.push(variable); 
        },

        syn::Pat::Type(pat_type) => {
            let type_str = pat_type.ty.to_token_stream().to_string();
            
            // Если это кортеж с типом
            if let syn::Type::Tuple(type_tuple) = &*pat_type.ty {
                // Для каждого элемента кортежа свой тип
                if let syn::Pat::Tuple(pat_tuple) = &*pat_type.pat {
                    for (i, elem_pat) in pat_tuple.elems.iter().enumerate() {
                        let elem_type = type_tuple.elems.get(i).map(|t| t.to_token_stream().to_string());
                        parse_pat(elem_pat.clone(), elem_type, context);
                    }
                
                    return;
                }
            }
    
            parse_pat(*pat_type.pat, Some(type_str), context);
        },

        syn::Pat::Tuple(pat_tuple) => {
            for element in pat_tuple.elems.iter() {
                parse_pat(element.clone(), None, context);
            }
        },

        syn::Pat::Struct(pat_struct) => {
            for field in pat_struct.fields.iter() {
                parse_pat(*field.pat.clone(), None, context);
            }
        },

        syn::Pat::Slice(pat_slice) => {
            for (_index, element) in pat_slice.elems.iter().enumerate() {
                parse_pat(element.clone(), None, context);
            }
        },

        syn::Pat::Or(pat_or) => {
            for (_index, case) in pat_or.cases.iter().enumerate() {
                parse_pat(case.clone(), None, context);
            }
        },

        syn::Pat::Reference(pat_ref) => {
            parse_pat(*pat_ref.pat, current_type, context);
        },

        syn::Pat::Paren(pat_paren) => {
            parse_pat(*pat_paren.pat, current_type, context);
        },

        syn::Pat::TupleStruct(pat_tuple_struct) => {
            for element in pat_tuple_struct.elems.iter() {
                parse_pat(element.clone(), None, context);
            }
        },

        syn::Pat::Path(pat_path) => {
            // TODO: обработать path
        },

        syn::Pat::Macro(pat_macro) => {
            // TODO: обработать макрос
        },

        _ => {},
    };
}
