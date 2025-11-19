pub mod element;
pub mod layout;
pub mod signals;
pub mod prelude;

mod widget_tree;
mod moon_bridge;

use element::Element;
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub use moon_bridge::{MoonBridge, Runnable};
pub use widget_tree::{update_tree, FireTree};

#[allow(dead_code)]
static ROOT_FN: Lazy<Mutex<Option<Box<dyn Fn() -> Element + Send + 'static>>>> =
    Lazy::new(|| Mutex::new(None));

pub fn app<F>(app_func: F)
where
    F: Fn() -> Element + Send + 'static,
{
    #[cfg(target_os = "android")]
    {
        let mut root_fn_guard = ROOT_FN.lock().unwrap();
        *root_fn_guard = Some(Box::new(app_func));
    }

    #[cfg(not(target_os = "android"))]
    {
        let app_runner = moon_bridge::MoonBridge::new().expect("Failed to create MoonBridge");
        
        let root_element = app_func();
        
        let mut render_tree = FireTree::new();
        
        update_tree(&root_element, &mut render_tree, app_runner.bridge());
        
        app_runner.run();
    }
}

#[cfg(target_os = "android")]
#[ndk_glue::main(logger(level = "info", tag = "firework-app"))]
fn android_main() {
    let root_fn = ROOT_FN.lock().unwrap().take()
        .expect("firework::app() must be called from main() to register the UI function.");

    let window = ndk_glue::native_window();
    let asset_manager = ndk_glue::asset_manager();
    let moonwalk = moonwalk::MoonWalk::new_android(&window, asset_manager)
        .expect("Failed to create MoonWalk for Android");

    let bridge = moon_bridge::MoonBridge::from_moonwalk(moonwalk);
    let mut render_tree = FireTree::new();

    loop {
        match ndk_glue::poll_events() {
            Some(ndk_glue::Event::Draw) => {
                let root_element = root_fn();
                update_tree(&root_element, &mut render_tree, &bridge);

                let mut mw = bridge.moonwalk();
                
                mw.set_viewport(window.width() as u32, window.height() as u32);
                let _ = mw.render_frame(glam::Vec4::new(0.1, 0.2, 0.3, 1.0));
            }
            
            None => break,
            
            _ => {}
        }
    }
}