use firework_ui::ui;

#[ui]
fn test_screen() {
    let mut a = spark!(0);

    if a == 0 {
        let mut b = spark!(0);
    }

    println!("Hello world");
}

fn main() {
    firework_ui::run(test_screen);
}
