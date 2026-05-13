use firework_ui::ui;

/*
#[ui]
fn test_screen() {
    let mut my_state = spark!(0);

    effect!(my_state, {
        println!("Component mount");
    });

    my_state += 1;
}
*/

#[ui]
fn test_combine_nested_structures_screen() {
    /*
    vertical! {
        layout! {
            padding: (10, 10, 10, 10),
        }

        rect! {
            position: (10, 10),
            color: (255, 255, 255),
        }
    }
    */
    println!("Hello world");
}

fn main() {
    firework_ui::run_with_adapter(
        firework_ui::null_adapter,
        test_combine_nested_structures_screen,
    );
}
