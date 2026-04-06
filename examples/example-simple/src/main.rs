use firework::ui;

ui! {
    fn test_screen() { 
        let mut spark1 = spark!(0u32);

        if spark1 >= 1 && spark1 < 5 {
            println!("Реактивное обновление!");
            spark1 += 1;
        } else { 
            spark1 += 1;       
        }
    }
}

fn main() {
    firework::run(test_screen);
}
