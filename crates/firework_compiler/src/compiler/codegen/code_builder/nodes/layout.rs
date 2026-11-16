// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use crate::CompileType;

use super::super::*;

impl CodeBuilder {
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(span = ?span)))]
    pub fn node_layout(
        &mut self,
        span: Span,
        final_tokens: &mut TokenStream,
        statement: &FireworkStatement,
        visitor: &mut CodegenVisitor,
        processed_body: &TokenStream,
    ) -> bool {
        if let FireworkAction::LayoutBlock(name, _microruntime, descriptor, depth, children) =
            &statement.action
        {
            // Хелпер это инструмент разработчика, для него не нужно генерировать лайаут
            if name == "helper" {
                return false;
            }

            let depth = *depth;
            let is_component = matches!(visitor.flags.compile_type, CompileType::Component);

            // Локальные счётчики для этого лайаута
            let current_constraints = format_ident!("_fwc_constraints_{}", depth);
            let current_pos_x = format_ident!("_fwc_pos_x_{}", depth);
            let current_pos_y = format_ident!("_fwc_pos_y_{}", depth);

            let init_layout = if depth == 1 {
                let root_constraints_init = if is_component {
                    // Для компонента берём размер из его подложки, этот размер устаналивается
                    // пропом size (методом __set_size внутри). Нужно использовать именно эти
                    // размеры, а не глобальные размеры окна, так как компонент это только
                    // часть экрана или другого компонента
                    quote_spanned! {span=>
                        let (_fwc_root_w, _fwc_root_h) = if let Some(_c) = &self._fwc__fwc_component {
                            _c.size
                        } else {
                            (0, 0)
                        };
                    }
                } else {
                    // Для экрана запрашиваем размер у адаптера синхронно
                    quote_spanned! {span=>
                        let (_fwc_root_w, _fwc_root_h) = if let firework_ui::AdapterResult::Size(w, h) =
                            firework_ui::adapter_command(firework_ui::AdapterCommand::GetViewport)
                        {
                            (w as i32, h as i32)
                        } else {
                            (0, 0)
                        };
                    }
                };

                quote_spanned! {span=>
                    #root_constraints_init

                    let mut #current_constraints = firework_ui::layout::Constraints {
                        min_width: 0,
                        max_width: _fwc_root_w,
                        min_height: 0,
                        max_height: _fwc_root_h,
                    };
                    let mut #current_pos_x = 0;
                    let mut #current_pos_y = 0;
                }
            } else {
                // Для глубина лайаута в иерархии больше 1, то есть это вложенный лауаут, то
                // берем ограничения от предыдущего уровня глубины
                let previous_depth = depth - 1;
                let previous_constraints = format_ident!("_fwc_constraints_{}", previous_depth);
                let layout_size_ident = format_ident!("_fwc_layout_size_{}", depth);

                self.layout_declaration_tokens.push((
                    current_constraints.to_string(),
                    quote_spanned! {span=>
                        let mut #current_constraints = firework_ui::layout::Constraints {
                            min_width: 0, max_width: 0, min_height: 0, max_height: 0
                        };
                    },
                ));

                self.layout_declaration_tokens.push((
                    current_pos_x.to_string(),
                    quote_spanned! {span=> let mut #current_pos_x = 0; },
                ));

                self.layout_declaration_tokens.push((
                    current_pos_y.to_string(),
                    quote_spanned! {span=> let mut #current_pos_y = 0; },
                ));

                self.layout_declaration_tokens.push((
                    layout_size_ident.to_string(),
                    quote_spanned! {span=>
                        let mut #layout_size_ident = firework_ui::layout::Size { width: 0, height: 0 };
                    },
                ));

                quote_spanned! {span=>
                    #current_constraints = #previous_constraints;
                }
            };

            // Измерение (1 проход)
            let measure_tokens = self.generate_measure(
                span,
                statement.screen_index,
                depth,
                name,
                is_component,
                descriptor,
                children,
            );

            // Раскладка (2 проход)
            let arrange_tokens = self.generate_arrange(
                span,
                statement.screen_index,
                depth,
                name,
                is_component,
                descriptor,
                children,
            );

            if depth == 1 {
                let mut declarations_tokens = TokenStream::new();
                let mut seen_variables = Vec::new();
                for (var_name, decl) in self.layout_declaration_tokens.drain(..) {
                    if !seen_variables.contains(&var_name) {
                        seen_variables.push(var_name);
                        declarations_tokens.extend(decl);
                    }
                }

                final_tokens.extend(quote_spanned!(span=>
                    {
                        #declarations_tokens
                        #init_layout

                        #processed_body

                        // Расчёт размеров всех виджетов
                        #measure_tokens
                        #arrange_tokens
                    }
                ));
            } else {
                self.layout_arrange_tokens
                    .entry(depth)
                    .or_default()
                    .push_back(arrange_tokens);

                final_tokens.extend(quote_spanned!(span=>
                    #init_layout
                    {
                        #processed_body
                    }

                    #measure_tokens
                ));
            }

            return true;
        }

        false
    }
}
