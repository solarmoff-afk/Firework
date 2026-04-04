use firework::ui;
// use firework::component;

ui! {
    fn test_screen() { 
        let mut a = spark!(5);

        vertical! {
            if a > 5 {
                println!("Реактивный блок");
            } else {
                a += 1;
            }
        }

        println!("Hello");
    } 
}

fn main() {
    test_screen();
}
