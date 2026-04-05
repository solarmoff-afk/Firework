use firework::ui;
// use firework::component;

ui! {
    fn test_screen() { 
        let mut spark1 = spark!(0u32);

        if spark1 > 1 && spark1 < 5 {
            println!("Реактивное обновление!");
        } else if spark1 < 5 {
            spark1 += 1;
        }
    } 
}

fn main() {
    test_screen();
}
