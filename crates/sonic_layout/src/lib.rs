// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

mod element;

use element::{Element, ContainerType, WidgetType, StackFrame};

pub struct Sonic {
    pub tree: Element,
    stack: Vec<StackFrame>,
}

impl Sonic {
    pub fn new() -> Self {
        Self {
            tree: Element::None,
            stack: Vec::new(),
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

    pub fn pop_container(&mut self) {
        if let Some(frame) = self.stack.pop() {
            let container = Element::Container(frame.container_type, frame.children);
            
            if let Some(parent) = self.stack.last_mut() { 
                parent.children.push(container);
            } else {
                self.tree = container;
            }
        }
    }

    pub fn add_widget(&mut self, widget_type: WidgetType) {
        let widget = Element::Widget(widget_type);
        
        if let Some(parent) = self.stack.last_mut() {
            parent.children.push(widget);
        } else {
            self.tree = widget;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sonic_test() {
        let mut sonic = Sonic::new();

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
    }
}
