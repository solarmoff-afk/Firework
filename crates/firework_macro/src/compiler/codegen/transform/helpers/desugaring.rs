// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use proc_macro2::Span;
use quote::quote;

use super::super::*;

impl CodegenVisitor<'_> {
    /// Десахаризирует #[read] у разделяемого состояния. Позволяет автоматически сгенерировать
    /// геттер для него по имени get_{}, где {} это имя разделяемого состояния
    pub fn desugar_shared_read(
        &self,
        span: Span,
        field_id: u128,
        field_name: &str,
        field_type: &Type,
        struct_id: u128,
    ) -> Item {
        let function_name = format_ident!("get_{}", field_name);
        let struct_name = format_ident!("APPLICATIONUIBLOCKSTRUCT{}_INSTANCE", struct_id);
        let field_access = format_ident!("spark_{}", field_id);
        let field_ident = format_ident!("{}", field_name);
        let build_name = format_ident!("_fwc_fn_build{}", struct_id);

        let is_multithread = cfg!(feature = "safety-multithread");

        let access_code = if is_multithread {
            quote! {
                let _fwc_mutex_guard = #struct_name.get()
                    .unwrap().lock().unwrap();
                let #field_ident = _fwc_mutex_guard.#field_access.as_ref().unwrap();
            }
        } else {
            quote! {
                let #field_ident = unsafe {
                    (*&raw const #struct_name).#field_access.as_ref().unwrap()
                };
            }
        };

        #[cfg(not(feature = "safety-multithread"))]
        let result = parse_quote_spanned!(span=>
            pub fn #function_name() -> &'static #field_type {
                #build_name();
                #access_code
                &#field_ident
            }
        );

        #[cfg(feature = "safety-multithread")]
        let result = parse_quote_spanned!(span=>
            pub fn #function_name() -> #field_type {
                #build_name();
                #access_code
                #field_ident.clone()
            }
        );

        result
    }

    /// Десахаризирует #[write] у разделямого состояния, генерирует сеттер под именем
    /// set_{}
    pub fn desugar_shared_write(
        &self,
        span: Span,
        field_id: u128,
        field_name: &str,
        field_type: &Type,
        struct_id: u128,
    ) -> Item {
        let function_name = format_ident!("set_{}", field_name);
        let struct_name = format_ident!("APPLICATIONUIBLOCKSTRUCT{}_INSTANCE", struct_id);
        let build_name = format_ident!("_fwc_fn_build{}", struct_id);
        let field_access = format_ident!("spark_{}", field_id);
        let new_value = format_ident!("new_{}", field_name);
        let field_ident = format_ident!("{}", field_name);

        // Запущен ли компилятор в режиме безопасной многопоточности
        let is_multithread = cfg!(feature = "safety-multithread");

        let access_code = if is_multithread {
            quote! {
                let mut _fwc_mutex_guard = #struct_name.get()
                    .unwrap().lock().unwrap();
                let #field_ident = _fwc_mutex_guard.#field_access.as_mut().unwrap();
            }
        } else {
            quote! {
                let mut #field_ident = unsafe {
                    (*&raw mut #struct_name)
                        .#field_access.as_mut().unwrap()
                };
            }
        };

        let temp = Vec::new();
        let func_effects = self.ir.shared.effects.get(field_name).unwrap_or(&temp);
        let mut func_effects_statements = Vec::new();

        for effect in func_effects {
            let ident = format_ident!("{}", effect);

            func_effects_statements.push(quote! {
                #ident();
            });
        }

        parse_quote_spanned!(span=>
            pub fn #function_name(#new_value: #field_type) {
                #build_name();

                {
                    #access_code
                    *#field_ident = #new_value;
                }

                #(#func_effects_statements)*
            }
        )
    }
}
