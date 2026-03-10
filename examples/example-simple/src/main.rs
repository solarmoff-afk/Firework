use firework::ui;

fn main() {
    ui! {
        let signal = signal!(10);

        {
            rect!()
                .width_percent(100);
        }

        if signal > 9 {
            text!("Hello world!").font_size(signal);
        }

        signal = 5;
    }
}
