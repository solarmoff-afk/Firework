mod moon_bridge;
pub use moon_bridge::MoonBridge;

use glam::Vec4;

pub mod element;
use element::Element;

pub mod prelude;

pub fn app<F>(app_func: F)
where
    F: Fn() -> Element + Send + 'static,
{
    let root_element = app_func();
    println!("{:#?}", root_element);

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