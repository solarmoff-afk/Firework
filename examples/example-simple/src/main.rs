use firework::ui;
// use firework::component;

ui! {
    fn test_screen() {
        let mut b: Vec<u32> = spark!(10); 
    }
}

fn main() {
    test_screen();
}
