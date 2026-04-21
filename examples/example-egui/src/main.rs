mod adapter;

use firework_ui::ui;
use firework_ui::{AdapterCommand, AdapterResult};
use adapter::egui_adapter;

#[ui]
fn test_screen() { 
    let mut spark1 = spark!(0u32);
    let mut spark2 = spark!(0u32);
    
    effect!({
        let rect = firework_ui::DefaultRectSkin::new(1)
            .unwrap()
            .position(10, 10)
            .size(100, 100)
            .color(255, 0, 0)
            .z(5)
            .visible(true)
            .hit_group(1);
    });

    spark1 += spark2;
    
    effect!(spark1, {
        println!("Update spark1: {}", spark1);
        spark2 = 10;
    });
    
    spark2 = 10;
}

fn main() {
    firework_ui::run_with_adapter(egui_adapter, test_screen);
}
