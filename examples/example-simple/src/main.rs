use firework_ui::ui;

#[ui]
fn test_screen() {
    let mut my_state = spark!(0);

    effect!(my_state, {
        println!("Component mount");
    });

    my_state += 1;
}


fn main() {
    firework_ui::run(test_screen);
}
