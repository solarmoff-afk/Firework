use firework_ui::ui;
use firework_ui::{AdapterCommand, AdapterResult};

ui! {
    fn test_screen() { 
        let mut spark1 = spark!(0u32);
        let mut spark2 = spark!(0u32);
        
        spark1 += spark2;

        effect!(spark1, {
            println!("Update spark1: {}", spark1);
            spark2 = 10;
        });

        spark2 = 10;
    }
}

fn my_adapter(command: AdapterCommand) -> AdapterResult {
    match command {
        AdapterCommand::RemoveAll => {
            println!("Remove all");
        },

        AdapterCommand::RunLoop { title, width, height, .. } => {
            println!("Run loop, {}, {}, {}", title, width, height);
        },
    }

    AdapterResult::Void
}

fn main() {
    // firework_ui::run(test_screen);
    firework_ui::run_with_adapter(my_adapter, test_screen);
}
