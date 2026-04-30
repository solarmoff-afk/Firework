use firework_ui::{ui, component, Prop, ComponentContext};

component! {
    pub struct Button {
        pub example_prop: Prop<bool>,
    }

    impl Button {
        pub fn new() -> Self {
            Self {
                example_prop: None,
            }
        }

        pub fn flash(&self, context: ComponentContext) {
            effect!({
                println!("Component mount");
            });
        }
    }
}

/*
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
