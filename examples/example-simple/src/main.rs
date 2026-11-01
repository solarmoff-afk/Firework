use firework_ui::ui;

#[ui]
fn test_screen() {
    let mut red = spark!(0u8);

    horizontal! {
        horizontal! {
            timer! {
                target: red,
                output: 200,
                delay: 3000,
            }
        }
    }
}

fn main() {
    firework_ui::run_with_adapter(firework_ui::null_adapter, test_screen);
}
