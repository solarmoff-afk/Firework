use firework_ui::ui;

ui! {
    fn test_screen() {
        println!("Hello world");
    }
}

fn main() {
    firework_ui::run(test_screen);
}
