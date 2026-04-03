// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub mod template;
pub mod element;

pub use template::SonicTemplate;

use element::{Element, ElementKind, ContainerType, WidgetType, StackFrame};
use template::replace_placeholders;

pub struct Sonic {
    pub tree: Element, 
    stack: Vec<StackFrame>,
    template: SonicTemplate,
    counter: usize,
}

impl Sonic {
    pub fn new(template: SonicTemplate) -> Self {
        Self {
            tree: Element::new(),
            stack: Vec::new(),
            template,
            counter: 0,
        }
    }

    pub fn log(&self) {
        println!("Stack: {:#?}\n\nTree: {:#?}", self.stack, self.tree);
    }

    pub fn push_container(&mut self, container_type: ContainerType) {
        self.stack.push(StackFrame {
            container_type,
            children: Vec::new(),
        });
    }

    pub fn pop_container(&mut self) -> usize {
        if let Some(frame) = self.stack.pop() {
            let container = Element {
                kind: ElementKind::Container(frame.container_type, frame.children),
                id: self.counter,
            };
            
            if let Some(parent) = self.stack.last_mut() { 
                parent.children.push(container);
            } else {
                self.tree = container;
            }
        }

        self.counter += 1;
        self.counter
    }

    pub fn add_widget(&mut self, widget_type: WidgetType) -> usize {
        let widget = Element {
            kind: ElementKind::Widget(widget_type),
            id: self.counter,
        };
        
        if let Some(parent) = self.stack.last_mut() {
            parent.children.push(widget);
        } else {
            self.tree = widget;
        }

        self.counter += 1;
        self.counter
    }

    pub fn genenerate_buffers(&self) -> String {
        

        "".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sonic_test() {
        let mut sonic = Sonic::new(SonicTemplate {
            layout_variable: "\tpub _fwc_layout_{name}: {type},".to_string(),
        });

        sonic.push_container(ContainerType::Vertical);
            sonic.log();

            sonic.add_widget(WidgetType::Fixed);
            sonic.add_widget(WidgetType::Fixed);
            sonic.add_widget(WidgetType::Fixed);

            sonic.push_container(ContainerType::Horizontal);
                sonic.add_widget(WidgetType::Fixed);
            sonic.pop_container();
        sonic.pop_container();

        sonic.log();

        println!("{}", sonic.genenerate_buffers());
    }
}
