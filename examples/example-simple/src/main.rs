use firework::ui;
// use firework::component;

ui! {
    // static BAD_STATIC: u32 = spark!(200);

    fn test_screen(test: u32) {
        {
            let mut spark1: u32 = spark!(0);
            spark1.push(10);
        }

        if a == b {
            let mut spark1: u32 = spark!(0);
        }
    }
}

fn main() {
    test_screen(0);
}
