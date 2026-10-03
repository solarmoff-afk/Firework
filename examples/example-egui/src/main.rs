mod adapter;

use adapter::egui_adapter;
use firework_ui::{BuildContext, Prop, component, ui};

/*
#[ui]
fn test_screen() {
    let mut rect_state = spark!(true);

    if rect_state {
        text! {
            text: "Hello world",
            position: (10, 10),
            color: (0, 255, 0),
        }
    }

    // rect_state = false;
}
*/

/*
#[ui]
fn test_screen() {
    let mut rect_state = spark!(true);

    if rect_state {
        rect! {
            position: (10, 10),
            width: 100,
            height: 100,
            color: (0, 255, 0),
            on_click: || {
                println!("Hello 4");
                rect_state = false;
            },
        }
    }

    // rect_state = false;
}
*/

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
            self.hello = T::default();

            rect! {
                position: (0, 0),
                size: 100,
                color: self.color.unwrap_or((255, 255, 255)),
            }
        }
    }
}

#[ui]
fn test_screen() {
    let mut x = spark!(10);
    let mut r = spark!(0u8);

    rect! {
        position: (10, 10),
        size: 100,
        color: (0, 255, 0),
        on_click: || {
            x += 25;
        },
    }

    rect! {
        position: (150, 10),
        size: 100,
        color: (255, 0, 0),
        on_click: || {
            x -= 25;
        },
    }

    component! {
        target: Button,
        T: i32,
        position: (290, 10),
        size: 100,
        color: (r, 0, 255),
        on_click: || {
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

/*
#[ui]
fn test_screen() {
    let mut x = spark!(3);

    for i in 0..x {
        for j in 0..3 {
            rect! {
                position: (150 * i, 150 * j),
                size: (100, 100),
                color: (0, 255, 0),

                #[key_type((i32, i32))]
                key: (i, j),

                /*
                on_click: || {
                    println!("Hello world");
                }
                */
            }
        }
    }

    x += 1;
}
*/

/*
#[ui]
fn test_screen() {
    /*
    let mut state: i32 = spark!(0, async move |mut bridge| {
        bridge.sleep_s(2);
        *bridge = 1;
        bridge.sleep_s(2);
    });
    */

    println!("Hello world");
}
*/

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
