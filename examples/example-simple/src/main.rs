use firework::ui;
// use firework::component;

ui! {
    fn test_screen() { 
        'loop: for i in 0..3 {
            let mut spark1 = spark!(0u32);

            'sub_loop: for j in 0..3 {
                func(j);
                break 'loop;
            }
        }
    } 
}

fn main() {
    test_screen();
}
