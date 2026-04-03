// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use crate::{Element, ElementKind, SonicTemplate, replace_placeholders, ContainerType};

/// Генератор кода лайаута
pub(crate) struct LayoutGenerator { 
    /// Поле для хранения айди последнего контейнера и его типа, это нужно чтобы получить
    /// переменные курсора и других полей лайаута и узнать контекст
    last_container_id: (usize, ContainerType),
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
        self.process_element(element, template, &mut output);
        
        output
    }
    
    fn process_element(&mut self, element: &Element, template: &SonicTemplate, output: &mut String) {
        match &element.kind {
            ElementKind::Container(container_type, children) => {
                // Сохранянение последнего айди контейнера чтобы определить название
                // переменной курсора
                self.last_container_id = (element.id, container_type.clone());

                match container_type {
                    ContainerType::Vertical => {
                        println!("Vertical");
                    },
                    
                    _ => todo!(),
                };
                
                for child in children {
                    self.process_element(&child, template, output);
                }
            },
            
            _ => {},
        }
    }
}
