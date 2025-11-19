use firework::prelude::*;

fn main() {
    app(root);
}

fn root() -> Element {
    let version = "0.0.1";

    container![
        text!("Welcome to Firework!").color(Color::WHITE),
        text!("Version: {}", version),
        rect!(15).color(Color::BLUE).position((300.0, 200.0)),
    ]
}