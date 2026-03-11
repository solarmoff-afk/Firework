use firework::ui;

ui! {
    fn test_screen() { 
        let spark1 = spark!(100);

        // rect!().width(spark1);
    }
}

fn main() {
    test_screen();
}
