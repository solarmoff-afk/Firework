use firework_ui::shared;

// Service
shared! {
    pub fn example(value: i32) {
        println!("Hello world: {}", value);
    }

    pub fn example2(value: i32) {
        println!("Hello world 2: {}", value * 2);
    }
}

fn main() {
    example(1);
    example2(10);
}
