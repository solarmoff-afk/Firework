mod adapter;

use firework_ui::ui;
use firework_ui::{AdapterCommand, AdapterResult};
use adapter::egui_adapter;

#[ui]
fn test_screen() { 
    let mut spark1 = spark!(0u32);
    let mut spark2 = spark!(0u32);
    
    effect!({
        let result = egui_adapter(AdapterCommand::NewRect { layout: 0 });

        if let AdapterResult::Handle(id) = result { 
            egui_adapter(AdapterCommand::SetPosition(id, (100, 100)));
            egui_adapter(AdapterCommand::SetSize(id, (200, 50)));
            egui_adapter(AdapterCommand::SetColor(id, (0, 0, 255, 255))); 
        }
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
