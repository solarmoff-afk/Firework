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
    let mut a = spark!(10);
    let mut b = spark!(10);

    let mut change1 = || a += 1;
    let mut change2 = || b += 1;

    change1();
    change2();
}

fn main() {
    firework_ui::run_with_adapter(
        firework_ui::null_adapter,
        test_combine_nested_structures_screen,
    );
}
