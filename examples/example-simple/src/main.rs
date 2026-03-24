use firework::ui;
// use firework::component;

ui! {
    struct a {
        b: u32,
    }

    fn test_screen() {
        let mut b: Vec<u32> = spark!(10);

        rect!{
            on_click: || {
                b += 1;
                println!("Hi");
            },
        }
    }

    fn test_screen2() {
        let mut b: Vec<u32> = spark!(10);

        rect!{
            on_click: || {
                b += 1;
                println!("Hi");
            },
        }
    }
}

fn main() {
    test_screen();
}
