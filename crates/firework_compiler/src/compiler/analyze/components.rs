// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub use super::*;

impl<'ast> Analyzer {
    pub(crate) fn validate_flash_signature(&mut self, method: &'ast ImplItemFn) -> Option<String> {
        let sig = &method.sig;
        let mut context_name: Option<String> = None;

        // FE020: Нельзя использовать unsafe в сигнатуре флэша компонента
        if let Some(unsafe_token) = sig.unsafety {
            self.context.errors.push(compile_error_spanned(
                unsafe_token,
                COMPONENT_FLASH_UNSAFE_ERROR,
            ));
        }

        // FE021: Флэш не может возвращать значение
        if !matches!(sig.output, ReturnType::Default) {
            self.context.errors.push(compile_error_spanned(
                &sig.output,
                COMPONENT_FLASH_RETURN_ERROR,
            ));
        }

        let mut inputs = sig.inputs.iter().enumerate();

        // FE022: Флэш должен иметь &mut self первым аргументом
        match inputs.next() {
            Some((_, FnArg::Receiver(recv))) => {
                if recv.reference.is_none() || recv.mutability.is_none() {
                    self.context
                        .errors
                        .push(compile_error_spanned(recv, COMPONENT_FLASH_MUT_SELF_ERROR));
                }
            }

            Some((_, arg)) => {
                self.context
                    .errors
                    .push(compile_error_spanned(arg, COMPONENT_FLASH_MUT_SELF_ERROR));
            }

            None => {
                self.context
                    .errors
                    .push(compile_error_spanned(sig, COMPONENT_FLASH_MUT_SELF_ERROR));
            }
        }

        // FE023: Флэш компонента должен иметь контекст
        match inputs.next() {
            Some((_, FnArg::Typed(pat_type))) => {
                let ty_str = pat_type.ty.to_token_stream().to_string().replace(" ", "");

                if ty_str.contains("ComponentContext") {
                    if let Pat::Ident(ref id) = *pat_type.pat {
                        context_name = Some(id.ident.to_string());
                    }
                } else {
                    self.context.errors.push(compile_error_spanned(
                        pat_type,
                        COMPONENT_FLASH_CONTEXT_MISSING_ERROR,
                    ));
                }
            }

            _ => {
                self.context.errors.push(compile_error_spanned(
                    sig,
                    COMPONENT_FLASH_CONTEXT_MISSING_ERROR,
                ));
            }
        }

        // FE024, FE025
        for (_, arg) in inputs {
            if let FnArg::Typed(pat_type) = arg {
                let raw_type_name = pat_type.ty.to_token_stream().to_string();
                let clean_type = raw_type_name.replace(" ", "");

                // Если контекст не вторым аргументом
                let is_context = clean_type.contains("ComponentContext")
                    || clean_type.contains("firework_ui::ComponentContext");

                if is_context {
                    self.context.errors.push(compile_error_spanned(
                        pat_type,
                        COMPONENT_FLASH_MULTIPLE_CONTEXT_ERROR,
                    ));

                    continue;
                }

                // Проверка на Prop<T>
                let is_prop =
                    clean_type.starts_with("Prop<") || clean_type.starts_with("firework_ui::Prop<");

                if is_prop {
                    continue;
                }

                // FE024: Невалидный тип пропса
                let msg = COMPONENT_FLASH_INVALID_ARG_ERROR.replace("{}", &raw_type_name);
                self.context
                    .errors
                    .push(compile_error_spanned(&pat_type.ty, &msg));
            }
        }

        context_name
    }
}
