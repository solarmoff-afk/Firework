use firework_ui::{BuildContext, Prop, component, ui};

component! {
    pub struct ForTimer {
        pub example_prop: Prop<bool>,
    }

    impl ForTimer {
        pub fn new() -> Self {
            Self {
                example_prop: None,
            }
        }

        pub fn flash(&mut self, _context: BuildContext) {
            let mut my_state = spark!(0);

            timer! {
                target: my_state,
                output: 123,
                delay: 2500,
            }

            effect!(my_state, {
                println!("State: {}", my_state);
            });
        }
    }
}

#[ui]
fn test_screen() {
    let mut red = spark!(0u8);

    component! {
        target: ForTimer,
    }

    timer! {
        target: red,
        output: 100,
        delay: 1500,
        on_timeout: || { println!("Now"); },
    }

    timer! {
        target: red,
        output: 200,
        delay: 3000,
    }

    effect!(red, {
        println!("Red: {}", red);
    });
}

fn main() {
    firework_ui::run_with_adapter(firework_ui::null_adapter, test_screen);
}
