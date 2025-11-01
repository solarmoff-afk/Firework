use firework::{run, hello_firework};

fn main() {
    run(|| {
        hello_firework()
    });
}