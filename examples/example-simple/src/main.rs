use firework::ui;
// use firework::component;

ui! {
    fn test_screen() {
        let mut b: Vec<u32> = spark!(10);
        
        vertical! {
            layout! {
                field1: b,
                field2: 20,
            };

            stack! {
                layout! {
                    field1: b,
                    field2: 20,
                };
            }
        };

        b = 5;
        b += 1;
        b.test.field = 1;
        // b.len();

        if b == 5 {
            b.len();
            a = b;

            if b == 5 {
                c = b;
            }
        }

        b = c;
    }
}

fn main() {
    test_screen();
}
