use firework_ui::{BuildContext, Prop, component};

component! {
    pub struct Button<T> {
        pub color: Prop<(u8, u8, u8)>,
        pub hello: T,
    }

    impl<T: Default> Button<T> {
        pub fn new() -> Self {
            Self {
                color: None,
                hello: T::default(),
            }
        }

        pub fn flash(&mut self, _context: BuildContext) {
            let mut test = spark!(10);

            self.color = Some((1, 1, 1));

            if self.color.is_some() {
                println!("Hello");
            }

            rect! {
                position: (test, 0),
                size: 100,
                color: self.color.unwrap_or((255, 255, 255)),
            }

            test = 5;
        }
    }
}

fn main() {
    // firework_ui::run_with_adapter(firework_ui::null_adapter, test_screen);
}
