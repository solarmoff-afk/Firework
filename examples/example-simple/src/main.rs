use firework::ui;
// use firework::component;

ui! {
    fn test_screen() {
        let mut a: Vec<u32> = spark!(Vec::new());
        let mut b: Vec<u32> = spark!(Vec::new());
        (b).push(10);
    }
}

fn main() {
    test_screen();
}
