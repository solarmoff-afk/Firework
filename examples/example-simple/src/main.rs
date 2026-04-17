use firework_ui::ui;

#[ui]
fn test_screen() {
    let mut a = spark!(0);
    let mut b = spark!(0);

    a = b + 10;
    b = a * 10;
}

fn main() {
    firework_ui::run(test_screen);
}
