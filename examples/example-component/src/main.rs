use firework_ui::{ComponentContext, Prop, component};

component! {
    pub struct Button {
        pub example_prop: Prop<bool>,
        pub hello: i32,
    }

    impl Button {
        pub fn new() -> Self {
            Self {
                example_prop: None,
                hello: 0,
            }
        }

        pub fn flash(&mut self, _context: ComponentContext) {
            let mut my_state = spark!(123);

            effect!(my_state, {
                println!("Component mount");
            });

            my_state += 1;
            // my_state += self.example_prop;
        }
    }
}

/*
use firework_ui::ui;
#[ui]
fn test_screen() {
    let mut my_spark = spark!(0);
    let mut spark2 = spark!(10);

    my_spark = spark2 * 2;

    if my_spark > 0 {
        println!("Больше 0, {}", my_spark);
    }

    spark2 = 20;
}
*/

fn main() {
    // firework_ui::run(test_screen);
}
