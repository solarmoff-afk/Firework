use glam::{Vec4, Vec2};
use moonwalk::{MoonWalk, ObjectId};
use std::sync::{Arc, Mutex};

#[cfg(not(target_os = "android"))]
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub struct MoonBridge {
    moonwalk: Arc<Mutex<MoonWalk>>,
    pub scale_factor: f32,
}

pub trait Runnable {
    fn bridge(&self) -> &MoonBridge;
    fn run<F>(self, on_rebuild: F)
    where
        F: FnMut(&MoonBridge) + 'static;
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
            .with_title("Firework app")
            .with_inner_size(winit::dpi::LogicalSize::new(600, 800))
            .build(&event_loop)?;

        let scale_factor = window.scale_factor() as f32;

        let window_box = Box::new(window);
        let leaked_window: &'static Window = Box::leak(window_box);

        let moonwalk = MoonWalk::new(leaked_window)?;
        let bridge = MoonBridge {
            moonwalk: Arc::new(Mutex::new(moonwalk)),
            scale_factor,
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
            
            /*
                Это значение временное! Firework пока не
                поддерживает android, поэтому я сделал
                заглушку в хардкод число 2.5 :/
            */

            scale_factor: 2.5,
        }
    }

    pub fn moonwalk(&self) -> std::sync::MutexGuard<'_, MoonWalk> {
        self.moonwalk.lock().unwrap()
    }

    pub fn set_scale_factor(&mut self, factor: f32) {
        self.scale_factor = factor;
    }

    pub fn config_position_dp(&self, mw: &mut MoonWalk, id: ObjectId, position_dp: Vec2) {
        let position_px = position_dp * self.scale_factor;
        mw.config_position(id, position_px);
    }

    pub fn config_size_dp(&self, mw: &mut MoonWalk, id: ObjectId, size_dp: Vec2) {
        let size_px = size_dp * self.scale_factor;
        mw.config_size(id, size_px);
    }
}

#[cfg(not(target_os = "android"))]
impl Runnable for DesktopApp {
    fn bridge(&self) -> &MoonBridge {
        &self.bridge
    }

    fn run<F>(self, mut on_rebuild: F)
    where
        F: FnMut(&MoonBridge) + 'static,
    {
        let moonwalk = self.bridge.moonwalk.clone();
        let window_ptr = self.window_ptr;

        let mut bridge = self.bridge;

        self.event_loop
            .run(move |event, elwt| {
                elwt.set_control_flow(ControlFlow::Wait);

                match event {
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        
                        WindowEvent::Resized(size) => {
                            moonwalk.lock().unwrap().set_viewport(size.width, size.height);
                        },

                        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                            bridge.set_scale_factor(scale_factor as f32);
                            on_rebuild(&bridge);
                        },

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