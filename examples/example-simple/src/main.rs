use firework::ui;

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
    }
}

fn main() {
    firework::run(test_screen);
}
