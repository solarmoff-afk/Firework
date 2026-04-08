use firework_ui::ui;

ui! {
    fn test_screen() { 
        let mut spark1 = spark!(0u32);

        if spark1 >= 1 && spark1 < 5 {
            println!("Реактивное обновление!");
            spark1 += 1;
        } else if spark1 < 5 { 
            spark1 += 1;       
        }

        effect!(spark1, {
            println!("Привет мир");
        });

        spark1 += 1;
    }
}

fn main() {
    firework_ui::run(test_screen);
}
