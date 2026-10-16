mod adapter;

use adapter::egui_adapter;
use firework_ui::{BuildContext, Prop, component, raw, ui};

component! {
    pub struct Button<T> {
        pub color: Prop<(u8, u8, u8)>,
        pub hello: T,
    }

    impl<T: Default> Button<T> {
        pub fn new() -> Self {
            Self {
                color: None,
                hello: T::default(),
            }
        }

        pub fn flash(&mut self, _context: BuildContext) {
            rect! {
                position: (0, 0),
                size: 100,
                color: self.color.unwrap_or((255, 255, 255)),
                rounded: 16,
                on_click: || { println!("Hello") },
            }
        }
    }
}

#[ui]
fn test_screen() {
    let mut x = spark!(10);
    let mut red = spark!(0u8);

    rect! {
        position: (10, 10),
        size: 100,
        color: (0, 255, 0),
        rounded: 16,
        on_click: || {
            x += 25;
        },
    }

    rect! {
        position: (150, 10),
        size: 100,
        color: (255, 0, 0),
        rounded: 16,
        on_click: || {
            x -= 25;
        },
    }

    component! {
        target: Button,
        T: i32,
        position: (290, 10),
        size: 100,
        color: (red, 0, 255),
        on_click: || {
            timer! {
                target: red,
                output: raw!(red + 25),
                delay: 3000,
            }

            println!("Component click");
        }
    }

    text! {
        text: "Firework",
        position: (x, 200),
        color: (255, 165, 0),
        font_size: 72,
        on_click: || {
            println!("Hello world");
        }
    }
}

fn main() {
    firework_ui::run_with_adapter(egui_adapter, test_screen);
}

/*
// use firework_devtools::visual_dt::{init_visual_devtool, visual_devtool_adapter};

// Включение визуального инспектора
fn main() {
    init_visual_devtool(egui_adapter);
    firework_ui::run_with_adapter(visual_devtool_adapter, test_screen);
}
*/
