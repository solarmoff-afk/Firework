use firework_ui::ui;

#[ui]
fn test_screen() {
    let mut red = spark!(0u8);

    timer! {
        target: red,
        output: 100,
        delay: 1500,
    }

    timer! {
        target: red,
        output: 200,
        delay: 5000,
    }

    effect!(red, {
        println!("Red: {}", red);
    });
}

fn main() {
    firework_ui::run_with_adapter(firework_ui::null_adapter, test_screen);
}
