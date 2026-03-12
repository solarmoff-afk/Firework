use firework::ui;

ui! {
    fn test_screen() { 
        // let mut spark1 = 0;

        let mut spark1: u32 = spark!(0); // Statement 0

        spark1 = 2; // Statement 1 

        spark1.push(1); // Statement 2

        if spark1 == 5 { // Statement 3
            println!("Hello world!"); // Statement 4
        }
    }
}

fn main() {
    test_screen();
}
