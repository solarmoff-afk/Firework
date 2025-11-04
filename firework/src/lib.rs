mod moon_bridge;
pub use moon_bridge::MoonBridge;

use glam::Vec4;

pub fn run<F>(app: F)
where
    F: Fn() -> String + Send + 'static,
{
    let content = app();
    println!("Firework started");
    println!("UI content: {}", content);

    let bridge = MoonBridge::new().expect("Failed to create MoonBridge");

    {
        let mut mw = bridge.moonwalk();
        let id = mw.new_rect();
        mw.config_position(id, glam::Vec2::new(100.0, 100.0));
        mw.config_size(id, glam::Vec2::new(200.0, 150.0));
        mw.config_color(id, Vec4::new(1.0, 0.0, 0.0, 1.0));
    }

    bridge.run();
}

pub fn hello_firework() -> String {
    "Hello, firework!".to_string()
}