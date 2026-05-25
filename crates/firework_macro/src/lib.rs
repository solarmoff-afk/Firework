// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

extern crate proc_macro;

use firework_compiler::*;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Fields, Ident, ItemEnum, Type, parse_macro_input};

#[proc_macro]
pub fn shared(input: TokenStream) -> TokenStream {
    process_macro(input, CompileType::Shared, false)
}

#[proc_macro]
pub fn component(input: TokenStream) -> TokenStream {
    process_macro(input, CompileType::Component, false)
}

#[proc_macro]
pub fn ui_block(input: TokenStream) -> TokenStream {
    process_macro(input, CompileType::Screen, true)
}

#[proc_macro_attribute]
pub fn ui(_args: proc_macro::TokenStream, input: TokenStream) -> TokenStream {
    process_macro(input, CompileType::Screen, true)
}

fn process_macro(input: TokenStream, compile_type: CompileType, use_counter: bool) -> TokenStream {
    let input = parse_macro_input!(input);
    process_compile(input, compile_type, use_counter).into()
}

#[proc_macro_attribute]
pub fn effect(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    input
}

/// Макрос для firework_adapter который позволяет синхронизировать структуру адаптера и
/// структуру для тестов (так как интеграционные тесты хранят команды в статике, а лайфтайм
/// 'a делает это невозможным)
#[proc_macro_attribute]
pub fn sync_adapter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_enum = parse_macro_input!(item as ItemEnum);

    let original_item = input_enum.clone();

    let vis = &input_enum.vis;
    let enum_name = &input_enum.ident;
    let test_name = format_ident!("TestCommand");

    let (impl_generics, ty_generics, where_clause) = input_enum.generics.split_for_impl();

    let variants: Vec<_> = input_enum
        .variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.ident;
            match &variant.fields {
                Fields::Named(named) => {
                    let mut test_fields = Vec::new();
                    let mut match_patterns = Vec::new();
                    let mut construct_exprs = Vec::new();

                    for field in &named.named {
                        let name = field.ident.as_ref().unwrap();
                        let ty = &field.ty;

                        let mapped_ty = map_type(ty);
                        let mapped_expr = map_expr(name, ty);

                        test_fields.push(quote! { #name: #mapped_ty });
                        match_patterns.push(quote! { #name });
                        construct_exprs.push(quote! { #name: #mapped_expr });
                    }

                    (
                        quote! { #variant_name { #(#test_fields),* } },
                        quote! {
                            #enum_name::#variant_name { #(#match_patterns),* } =>
                            #test_name::#variant_name { #(#construct_exprs),* }
                        },
                    )
                }
                Fields::Unnamed(unnamed) => {
                    let mut test_fields = Vec::new();
                    let mut match_patterns = Vec::new();
                    let mut construct_exprs = Vec::new();

                    for (i, field) in unnamed.unnamed.iter().enumerate() {
                        let name = format_ident!("_{}", i);
                        let ty = &field.ty;

                        let mapped_ty = map_type(ty);
                        let mapped_expr = map_expr(&name, ty);

                        test_fields.push(mapped_ty);
                        match_patterns.push(quote! { #name });
                        construct_exprs.push(mapped_expr);
                    }

                    (
                        quote! { #variant_name(#(#test_fields),*) },
                        quote! {
                            #enum_name::#variant_name(#(#match_patterns),*) =>
                            #test_name::#variant_name(#(#construct_exprs),*)
                        },
                    )
                }
                Fields::Unit => (
                    quote! { #variant_name },
                    quote! { #enum_name::#variant_name => #test_name::#variant_name },
                ),
            }
        })
        .collect();

    let test_variants: Vec<_> = variants.iter().map(|(v, _)| v).collect();
    let from_arms: Vec<_> = variants.iter().map(|(_, arm)| arm).collect();

    let output = quote! {
        #original_item

        #[allow(unpredictable_function_pointer_comparisons)]
        #[derive(Debug, Clone, PartialEq)]
        #vis enum #test_name {
            #(#test_variants),*
        }

        impl #impl_generics From<#enum_name #ty_generics> for #test_name #where_clause {
            fn from(command: #enum_name #ty_generics) -> Self {
                match command {
                    #(#from_arms),*
                }
            }
        }
    };

    output.into()
}

fn is_string_ref(ty: &Type) -> bool {
    if let Type::Reference(type_ref) = ty
        && let Type::Path(type_path) = &*type_ref.elem
        && type_path.path.is_ident("str")
    {
        return true;
    }

    false
}

fn map_type(ty: &Type) -> proc_macro2::TokenStream {
    if is_string_ref(ty) {
        quote! { String }
    } else {
        quote! { #ty }
    }
}

fn map_expr(name: &Ident, ty: &Type) -> proc_macro2::TokenStream {
    if is_string_ref(ty) {
        quote! { #name.to_string() }
    } else {
        quote! { #name }
    }
}
