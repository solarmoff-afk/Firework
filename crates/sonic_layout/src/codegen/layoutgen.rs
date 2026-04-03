// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::layouts::utils::generate_set_variable;

use crate::codegen::layouts::utils::generate_add_variable;
use crate::{Element, ElementKind, SonicTemplate, replace_placeholders, ContainerType, WidgetType};

/// Генератор кода лайаута
pub(crate) struct LayoutGenerator { 
    /// Поле для хранения айди последнего контейнера и его типа, это нужно чтобы получить
    /// переменные курсора и других полей лайаута и узнать контекст
    pub last_container_id: (usize, ContainerType),
}

impl LayoutGenerator {
    pub fn new() -> Self {
        Self {
            // Значение по умолчанию для типа контейнера это абсолютный контейнер
            last_container_id: (0, ContainerType::Absolute),
        }
    }

    pub fn generate_layout(&mut self, element: &Element, template: &SonicTemplate) -> String {
        let mut output = "".to_string();
        self.process_element(element, template, &mut output, None);
        
        output
    }
    
    fn process_element(
        &mut self,
        element: &Element,
        template: &SonicTemplate,
        output: &mut String,
        parent_id: Option<(usize, ContainerType)>,
    ) {
        match &element.kind {
            ElementKind::Container(container_type, children) => {
                let my_id = element.id;
                let old_context = self.last_container_id.clone();

                self.last_container_id = (element.id, container_type.clone());

                match container_type {
                    ContainerType::Vertical => {
                        println!("Vertical");
                    },
                    
                    _ => todo!(),
                };

                let variable_name = format!("{}_total_size", element.id);
                generate_set_variable(&variable_name, "0.0", template, output);
                output.push('\n');

                let variable_name = format!("{}_fill_count", element.id);
                generate_set_variable(&variable_name, "0.0", template, output);
                output.push('\n');
                
                for child in children {
                    self.process_element(&child, template, output, Some((my_id, container_type.clone())));
                }

                if let Some((pid, p_type)) = parent_id.clone() {
                    match p_type {
                        ContainerType::Vertical => {
                            let my_val = replace_placeholders(
                                &template.get_layout_variable, "name",
                                &format!("{}_total_size", my_id)
                            );

                            generate_add_variable(&format!("{}_total_size", pid), &my_val, template, output);
                        },
                        
                        _ => todo!(),
                    }
                }

                self.last_container_id = old_context;
            },

            ElementKind::Widget(widget_type, id) => {
                match self.last_container_id.1 {
                    ContainerType::Vertical => {
                        println!("Vertical layout for widget");  
                        self.vertical_layout(template, output, *widget_type, *id);
                    },
                    
                    _ => todo!(),
                };
            },
            
            _ => {},
        }
    }
}
