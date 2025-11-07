pub mod element;
pub mod layout;
pub mod signals;
pub mod prelude;

mod widget_tree;
mod moon_bridge;

use element::Element;
use glam::Vec4;
pub use moon_bridge::MoonBridge;
pub use widget_tree::update_tree;

pub fn app<F>(app_func: F)
where
    F: Fn() -> Element + Send + 'static,
{
    let root_element = app_func();
    println!("{:#?}", root_element);

    update_tree(&root_element);

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