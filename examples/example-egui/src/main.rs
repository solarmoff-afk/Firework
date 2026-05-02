mod adapter;

use adapter::egui_adapter;
use firework_ui::ui;

/*
#[ui]
fn test_screen() {
    let mut rect_state = spark!(true);

    if rect_state {
        rect! {
            position: (10, 10),
            size: (100, 100),
            color: (0, 255, 0),
        }
    }

    rect_state = false;
}
*/

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

fn main() {
    firework_ui::run_with_adapter(egui_adapter, test_screen);
}
