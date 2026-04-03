// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::utils::{generate_add_position, generate_add_variable};

use crate::{LayoutGenerator, WidgetType};
use crate::{Element, ElementKind, SonicTemplate, replace_placeholders, ContainerType};

impl LayoutGenerator {
    pub fn vertical_layout(
        &mut self,
        template: &SonicTemplate,
        output: &mut String,
        widget_type: WidgetType,
        id: usize,
    ) {
        match widget_type {
            WidgetType::Fixed => {
                output.push_str(
                    format!("let (_fwc_sonic_w, _fwc_sonic_h) = {};\n", replace_placeholders(
                        &template.measure_widget,
                        "id",
                        &id.to_string(),
                    )
                ).as_str());

                generate_add_variable(
                    format!("{}_total_size", self.last_container_id.0).as_str(),
                    "_fwc_sonic_h",
                    template,
                    output
                );
            },

            _ => todo!(),
        };
    }
}
