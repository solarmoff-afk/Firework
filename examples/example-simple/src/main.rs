use firework::ui;

ui! {
    fn test_screen(test: u32) {
        {
            let mut spark1: u32 = spark!(0);
        }

        if a == b {
            let mut spark2: u32 = spark!(0);
        }

        // if let Some(x) = opt {
            let mut x: u32 = spark!(0);
        // }
    }
}

fn main() {
    test_screen(0);
}
