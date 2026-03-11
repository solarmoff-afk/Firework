use firework::ui;

fn test_screen() {
    ui! {
        let signal = signal!(123);
        let (signal1, signal2, b) = (signal!(1), signal!(256), 5);
        let mut signal3: u32 = signal!(12);

        rect!();

        let (x, y): (i32, i32) = (4, 5);

        vertical! {
            let test = 1;
            println!("hi!");
        }
    }
}

// Экран
// fn home() {
    // ui! {
        // let signal = signal!(10);
        //
        // let (signal1, signal2, b) = (signal!(1), signal!("Govno"), 5);
        //
        // {
        //    rect!()
        //        .width_percent(100);
        // }
        //
        // if signal > 9 {
        //    text!("Hello world!").font_size(signal);
        // }
        //
        // signal = 5;
    // }
// }

fn main() {
    // home();
    test_screen();
}
