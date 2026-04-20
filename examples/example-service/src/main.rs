use firework_ui::shared;

shared! {
    state! {
        counter: i32 = 0,
        counter2: i32 = 0,
    }

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
