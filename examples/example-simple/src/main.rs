use firework_ui::ui;

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

fn main() {
    firework_ui::run(test_screen);
}
