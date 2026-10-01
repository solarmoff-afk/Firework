use firework_ui::ui;

#[ui]
fn test_screen() {
    let mut count = spark!(3);

    for i in 0..count {
        rect! {
            position: (10, 10),
            color: (255, 255, 255),

            #[key_type(i32)]
            key: i,
        }
    }

    // Один виджет удаляется
    count -= 1;
}

fn main() {
    firework_ui::run_with_adapter(firework_ui::null_adapter, test_screen);
}
