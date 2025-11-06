use firework::prelude::*;

fn main() {
    app(root);
}

fn root() -> Element {
    let version = "0.0.1";

    container![
        text!("Welcome to Firework!").background(Color::WHITE),
        text!("Version: {}", version),
        rect!().background(Color::BLUE),
    ]
}