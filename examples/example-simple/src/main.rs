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
    firework_ui::run_with_adapter(firework_ui::null_adapter, test_screen);
}
