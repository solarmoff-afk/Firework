use firework::ui;

ui! {
    fn test_screen() {
        {
            let mut spark1: u32 = spark!(0);
        }

        if a == b {
            let mut spark2: u32 = spark!(0);
        }
    }
}

fn main() {
    test_screen();
}
