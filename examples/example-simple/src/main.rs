use firework::ui;
// use firework::component;

ui! {
    fn test_screen() { 
        for i in 1..5 {
            // Лафтайм A
            let mut a = spark!(0);

            if 1 == 1 {
                // Лайфтайм Б
                let mut b: u32 = spark!(0);

                if i == 1 {
                    // Лайфтайм А и лайфтайм Б заканчиваются
                    println!("Before");
                    break;
                    println!("After");
                }

                // Лайфтайм Б заканчивается 
            }

            // Лайфтайм А заканчивается 
        }
    } 
}

fn main() {
    test_screen();
}
