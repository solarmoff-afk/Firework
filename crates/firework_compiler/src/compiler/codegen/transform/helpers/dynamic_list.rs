// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

#[cfg(feature = "trace")]
use tracing::instrument;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote_spanned};

#[cfg_attr(feature = "trace", instrument(skip_all, fields(struct_name_raw = ?struct_name_raw)))]
pub fn generate_lifecycle(
    struct_name_raw: &str,
    dynamic_widgets: &[usize],
    span: Span,
) -> (TokenStream, TokenStream) {
    let mut begin_tokens = TokenStream::new();
    let mut end_tokens = TokenStream::new();

    let instance_ident_upper = format_ident!("{}_INSTANCE", struct_name_raw.to_uppercase());

    for widget_id in dynamic_widgets {
        let field_ident = format_ident!("_fwc_widget_object_{}", widget_id);

        #[cfg(feature = "safety-multithread")]
        {
            begin_tokens.extend(quote_spanned!(span=>
                {
                    let mut _fwc_inst = #instance_ident_upper.get()
                        .expect("Firework: Instance not initialized").lock().unwrap();

                    if _fwc_inst.#field_ident.is_none() {
                        _fwc_inst.#field_ident = Some(firework_ui::DynList::new());
                    }

                    _fwc_inst.#field_ident.as_mut().unwrap().begin_pass();
                }
            ));

            end_tokens.extend(quote_spanned!(span=>
                {
                    let mut _fwc_inst = #instance_ident_upper.get()
                        .expect("Firework: Instance not initialized").lock().unwrap();

                    if let Some(list) = _fwc_inst.#field_ident.as_mut() {
                        list.end_pass();
                    }
                }
            ));
        }

        #[cfg(not(feature = "safety-multithread"))]
        {
            begin_tokens.extend(quote_spanned!(span=>
                unsafe {
                    let inst = &mut *::core::ptr::addr_of_mut!(#instance_ident_upper);

                    if inst.#field_ident.is_none() {
                        inst.#field_ident = Some(firework_ui::DynList::new());
                    }

                    inst.#field_ident.as_mut().unwrap().begin_pass();
                }
            ));

            end_tokens.extend(quote_spanned!(span=>
                unsafe {
                    if let Some(list) = (
                        *::core::ptr::addr_of_mut!(#instance_ident_upper))
                        .#field_ident.as_mut()
                    {
                        list.end_pass();
                    }
                }
            ));
        }
    }

    (begin_tokens, end_tokens)
}
