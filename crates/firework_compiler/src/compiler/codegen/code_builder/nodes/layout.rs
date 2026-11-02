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
                let previous_pos_x = format_ident!("_fwc_pos_x_{}", previous_depth);
                let previous_pos_y = format_ident!("_fwc_pos_y_{}", previous_depth);

                quote_spanned! {span=>
                    let mut #current_constraints = #previous_constraints;
                    let mut #current_pos_x = #previous_pos_x;
                    let mut #current_pos_y = #previous_pos_y;
                }
            };

            let measure_tokens = self.generate_measure(
                span,
                statement.screen_index,
                depth,
                name,
                is_component,
                descriptor,
                children,
            );

            final_tokens.extend(quote_spanned!(span=>
                {
                    // println!("Layout begin placeholder");
                    #init_layout

                    #processed_body

                    // Расчёт размеров всех виджетов
                    #measure_tokens

                    // println!("Layout end placeholder");
                }
            ));

            return true;
        }

        false
    }
}
