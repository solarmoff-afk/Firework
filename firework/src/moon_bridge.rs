use glam::Vec4;
use moonwalk::MoonWalk;
use std::sync::{Arc, Mutex};

#[cfg(not(target_os = "android"))]
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub struct MoonBridge {
    moonwalk: Arc<Mutex<MoonWalk>>,
}

pub trait Runnable {
    fn bridge(&self) -> &MoonBridge;
    fn run(self);
}

#[cfg(not(target_os = "android"))]
pub struct DesktopApp {
    bridge: MoonBridge,
    event_loop: EventLoop<()>,
    window_ptr: *mut Window,
}

impl MoonBridge {
    #[cfg(not(target_os = "android"))]
    pub fn new() -> Result<DesktopApp, Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;
        let window = WindowBuilder::new()
            .with_title("Firework")
            .with_inner_size(winit::dpi::LogicalSize::new(600, 800))
            .build(&event_loop)?;

        let window_box = Box::new(window);
        let leaked_window: &'static Window = Box::leak(window_box);

        let moonwalk = MoonWalk::new(leaked_window)?;
        let bridge = MoonBridge {
            moonwalk: Arc::new(Mutex::new(moonwalk)),
        };

        Ok(DesktopApp {
            bridge,
            event_loop,
            window_ptr: leaked_window as *const Window as *mut Window,
        })
    }

    #[cfg(target_os = "android")]
    pub fn from_moonwalk(moonwalk: MoonWalk) -> Self {
        Self {
            moonwalk: Arc::new(Mutex::new(moonwalk)),
        }
    }

    pub fn moonwalk(&self) -> std::sync::MutexGuard<'_, MoonWalk> {
        self.moonwalk.lock().unwrap()
    }
}

#[cfg(not(target_os = "android"))]
impl Runnable for DesktopApp {
    fn bridge(&self) -> &MoonBridge {
        &self.bridge
    }

    fn run(self) {
        let moonwalk = self.bridge.moonwalk.clone();
        let window_ptr = self.window_ptr;

        self.event_loop
            .run(move |event, elwt| {
                elwt.set_control_flow(ControlFlow::Wait);

                match event {
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::Resized(size) => {
                            moonwalk.lock().unwrap().set_viewport(size.width, size.height);
                        }
                        _ => {}
                    },
                    
                    Event::AboutToWait => {
                        let _ = moonwalk
                            .lock()
                            .unwrap()
                            .render_frame(Vec4::new(0.1, 0.2, 0.3, 1.0));
                    }
                    
                    Event::LoopExiting => unsafe {
                        drop(Box::from_raw(window_ptr));
                    },
                    
                    _ => {}
                }
            })
            .expect("Event loop failed");
    }
}