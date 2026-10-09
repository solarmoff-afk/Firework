use firework_ui::ui;

#[ui]
fn compose() {
    /*
       let mut state = spark!(0, async move |mut bridge| {
            bridge.sleep_s(2);
            bridge.send(1);
            bridge.sleep_s(2);
        });

        effect!(state, {
            println!("Hello world");
        });
    */
}

fn main() {
    firework_ui::run_with_adapter(firework_ui::null_adapter, compose);
}
