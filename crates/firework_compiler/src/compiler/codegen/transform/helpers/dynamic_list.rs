// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

#[cfg(feature = "trace")]
use tracing::instrument;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote_spanned};

use crate::CompileType;
use crate::compiler::CodegenVisitor;

#[cfg_attr(feature = "trace", instrument(skip_all, fields(struct_name_raw = ?struct_name_raw)))]
pub fn generate_lifecycle(
    visitor: &mut CodegenVisitor,
    struct_name_raw: &str,
    dynamic_widgets: &[usize],
    span: Span,
) -> (TokenStream, TokenStream) {
    let is_component = matches!(visitor.flags.compile_type, CompileType::Component);

    let mut begin_tokens = TokenStream::new();
    let mut end_tokens = TokenStream::new();

    let instance_ident_upper = format_ident!("{}_INSTANCE", struct_name_raw.to_uppercase());

    for widget_id in dynamic_widgets {
        let field_ident = format_ident!("_fwc_widget_object_{}", widget_id);

        let begin_body = quote_spanned!(span=>
            if _fwc_target.#field_ident.is_none() {
                _fwc_target.#field_ident = Some(firework_ui::DynList::new());
            }

            _fwc_target.#field_ident.as_mut().unwrap().begin_pass();
        );

        let end_body = quote_spanned!(span=>
            if let Some(list) = _fwc_target.#field_ident.as_mut() {
                list.end_pass();
            }
        );

        #[cfg(feature = "safety")]
        {
            if is_component {
                begin_tokens.extend(quote_spanned!(span=>
                {
                    let _fwc_target = &mut *self;
                    #begin_body
                }
                ));
                end_tokens.extend(quote_spanned!(span=>
                {
                    let _fwc_target = &mut *self;
                    #end_body
                }
                ));
            } else {
                begin_tokens.extend(quote_spanned!(span=>
                {
                    #instance_ident_upper.with(|inst| {
                        let mut _fwc_inst = inst.borrow_mut();
                        let _fwc_target = &mut *_fwc_inst;
                        #begin_body
                    });
                }
                ));
                end_tokens.extend(quote_spanned!(span=>
                {
                    #instance_ident_upper.with(|inst| {
                        let mut _fwc_inst = inst.borrow_mut();
                        let _fwc_target = &mut *_fwc_inst;
                        #end_body
                    });
                }
                ));
            }
        }

        #[cfg(not(feature = "safety"))]
        {
            if is_component {
                begin_tokens.extend(quote_spanned!(span=>
                    {
                        let _fwc_target = &mut *self;
                        #begin_body
                    }
                ));

                end_tokens.extend(quote_spanned!(span=>
                    {
                        let _fwc_target = &mut *self;
                        #end_body
                    }
                ));
            } else {
                begin_tokens.extend(quote_spanned!(span=>
                    unsafe {
                        let _fwc_target = &mut *::core::ptr::addr_of_mut!(#instance_ident_upper);
                        #begin_body
                    }
                ));

                end_tokens.extend(quote_spanned!(span=>
                    unsafe {
                        let _fwc_target = &mut *::core::ptr::addr_of_mut!(#instance_ident_upper);
                        #end_body
                    }
                ));
            }
        }
    }

    (begin_tokens, end_tokens)
}
