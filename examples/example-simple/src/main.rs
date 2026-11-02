use firework_ui::{component, ui};

component! {
    pub struct BasicComponent {}

    impl BasicComponent {
        pub fn new() -> Self {
            Self {}
        }

        pub fn flash(&mut self, _context: BuildContext) {
            rect! {
                position: (10, 10),
                color: (255, 0, 0),
            }
        }
    }
}

#[ui]
fn test_screen() {
    let mut red = spark!(0u8);

    horizontal! {
        horizontal! {
            rect! {
                position: (100, 100),
                size: 100,
                color: (red, 0, 0),
            }
        }
    }
}

fn main() {
    firework_ui::run_with_adapter(firework_ui::null_adapter, test_screen);
}
