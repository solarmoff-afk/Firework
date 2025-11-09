pub mod element;
pub mod layout;
pub mod signals;
pub mod prelude;

mod widget_tree;
mod moon_bridge;

use element::Element;
use glam::Vec4;

pub use moon_bridge::MoonBridge;
pub use widget_tree::{update_tree, FireTree};

pub fn app<F>(app_func: F)
where
    F: Fn() -> Element + Send + 'static,
{
    let root_element = app_func();
    println!("{:#?}", root_element);

    let mut render_tree = FireTree::new();

    let bridge = MoonBridge::new().expect("Failed to create MoonBridge");

    update_tree(&root_element, &mut render_tree, &bridge);

    bridge.run();
}