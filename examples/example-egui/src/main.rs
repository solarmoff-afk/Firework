mod adapter;

use firework_ui::ui;
use adapter::egui_adapter;

/*
#[ui]
fn test_screen() { 
    let mut spark1 = spark!(10u32);
    let mut spark2 = spark!(0u32);

    rect! {
        position: (spark1.try_into().unwrap(), 10),
        size: (100, 100),
        color: (255, 0, 0),
    } 

    spark1 += spark2;
    
    effect!(spark1, {
        println!("Update spark1: {}", spark1);
        spark2 = 10;
    });
    
    spark2 = 10;
}
*/

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

    rect_state = true;
}

fn main() {
    firework_ui::run_with_adapter(egui_adapter, test_screen);
}
