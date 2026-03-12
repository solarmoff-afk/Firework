use firework::ui;

ui! {
    fn test_screen() {  
        let mut spark1: Vec<u32> = spark!(Vec::new()); // Statement 1
        let mut spark2: u32 = spark!(0); // Statement 2

        spark1 = 2; // Statement 3

        spark1.push(1); // Statement 4

        spark2 += 2; // Statement 5
        
        spark1.field = 10; // Statement 6

        if spark1 == 5 { // Statement 7
            println!("Hello world!"); // Statement 8
        }
    }
}

fn main() {
    test_screen();
}
