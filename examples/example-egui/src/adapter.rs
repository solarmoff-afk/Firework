// [AI GEN ADAPTER EXAMPLE FOR TEST]

use eframe::egui;
use firework_ui::{AdapterClickPhase, AdapterCommand, AdapterEvent, AdapterResult};
use std::sync::Mutex;

const MAX_OBJECTS: usize = 4096;

/// Внутреннее представление рендер-объекта
#[derive(Clone, Copy)]
struct RectObject {
    alive: bool,
    pos: (i32, i32),
    size: (i32, i32),
    color: (u8, u8, u8, u8),
    z_index: i32,
    visible: bool,
    hit_group: u16,
}

impl Default for RectObject {
    fn default() -> Self {
        Self {
            alive: false,
            pos: (0, 0),
            size: (0, 0),
            color: (255, 255, 255, 255),
            z_index: 0,
            visible: true,
            hit_group: 0,
        }
    }
}

/// Состояние адаптера
struct AdapterState {
    objects: [RectObject; MAX_OBJECTS],
    listener: Option<fn(AdapterEvent)>,
    dirty: bool,
}

static ADAPTER_STATE: Mutex<AdapterState> = Mutex::new(AdapterState {
    objects: [RectObject {
        alive: false,
        pos: (0, 0),
        size: (0, 0),
        color: (0, 0, 0, 0),
        z_index: 0,
        visible: false,
        hit_group: 0,
    }; MAX_OBJECTS],
    listener: None,
    dirty: false,
});

/// Главная функция адаптера
pub fn egui_adapter(cmd: AdapterCommand) -> AdapterResult {
    let mut state = ADAPTER_STATE.lock().unwrap();

    match cmd {
        AdapterCommand::RunLoop { title, width, height, listener } => {
            state.listener = Some(listener); 
            drop(state);

            let options = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_inner_size([width as f32, height as f32]),
                ..Default::default()
            };

            let _ = eframe::run_native(
                title,
                options,
                Box::new(|_cc| Ok(Box::new(FireworkEguiApp))),
            );

            AdapterResult::Void
        },

        AdapterCommand::RemoveAll => {
            for obj in state.objects.iter_mut() {
                obj.alive = false;
            }

            AdapterResult::Void
        },

        AdapterCommand::NewRect { layout: _ } => {
            if let Some(index) = state.objects.iter().position(|obj| !obj.alive) {
                state.objects[index] = RectObject {
                    alive: true,
                    visible: true,
                    ..Default::default()
                };
                state.dirty = true;

                AdapterResult::Handle(index)
            } else {
                AdapterResult::Fail
            }
        },

        AdapterCommand::SetPosition(id, pos) => {
            if let Some(obj) = state.objects.get_mut(id) {
                obj.pos = pos;
            }

            AdapterResult::Void
        },

        AdapterCommand::SetSize(id, size) => {
            if let Some(obj) = state.objects.get_mut(id) {
                obj.size = size;
            }

            AdapterResult::Void
        },

        AdapterCommand::SetColor(id, color) => {
            if let Some(obj) = state.objects.get_mut(id) {
                obj.color = color;
            }

            AdapterResult::Void
        },

        AdapterCommand::SetZ(id, z) => {
            if let Some(obj) = state.objects.get_mut(id) {
                obj.z_index = z;
                state.dirty = true;
            }

            AdapterResult::Void
        },

        AdapterCommand::SetVisible(id, visible) => {
            if let Some(obj) = state.objects.get_mut(id) {
                obj.visible = visible;
            }

            AdapterResult::Void
        },

        AdapterCommand::Remove(id) => {
            if let Some(obj) = state.objects.get_mut(id) {
                obj.alive = false;
                state.dirty = true;
            }

            AdapterResult::Void
        },

        AdapterCommand::SetHitGroup(id, group) => {
            if let Some(obj) = state.objects.get_mut(id) {
                obj.hit_group = group;
            }

            AdapterResult::Void
        },

        AdapterCommand::ResolveHit(group, (ax, ay, aw, ah)) => {
            let mut highest_z = i32::MIN;
            let mut found_id = None;

            let a_left = ax;
            let a_right = ax + aw;
            let a_top = ay;
            let a_bottom = ay + ah;

            for (id, obj) in state.objects.iter().enumerate() {
                if obj.alive && obj.visible && obj.hit_group == group { 
                    let b_left = obj.pos.0;
                    let b_right = obj.pos.0 + obj.size.0;
                    let b_top = obj.pos.1;
                    let b_bottom = obj.pos.1 + obj.size.1;

                    let intersects = a_left < b_right && 
                                     a_right > b_left && 
                                     a_top < b_bottom && 
                                     a_bottom > b_top;

                    if intersects && obj.z_index >= highest_z {
                        highest_z = obj.z_index;
                        found_id = Some(id);
                    }
                }
            }

            match found_id {
                Some(id) => AdapterResult::Handle(id),
                None => AdapterResult::Fail,
            }
        },

        AdapterCommand::Render => {
            AdapterResult::Void
        }
    }
}

struct FireworkEguiApp;

impl eframe::App for FireworkEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let listener = {
            let state = ADAPTER_STATE.lock().unwrap();
            state.listener
        };

        if let Some(listener) = listener {
            listener(AdapterEvent::Tick);

            if ctx.input(|i| i.viewport().close_requested()) {
                listener(AdapterEvent::CloseRequest);
            }

            ctx.input(|i| {
                if let Some(pos) = i.pointer.interact_pos() {
                    let x = pos.x.max(0.0) as u32;
                    let y = pos.y.max(0.0) as u32;

                    if i.pointer.any_pressed() {
                        listener(AdapterEvent::Touch(x, y, AdapterClickPhase::Began));
                    } else if i.pointer.any_released() {
                        listener(AdapterEvent::Touch(x, y, AdapterClickPhase::Ended));
                    } else if i.pointer.any_down() && i.pointer.delta() != egui::Vec2::ZERO {
                        listener(AdapterEvent::Touch(x, y, AdapterClickPhase::Moved));
                    }
                }

                for event in &i.events {
                    if let egui::Event::Text(text) = event {
                        if let Some(char_code) = text.chars().next() {
                            listener(AdapterEvent::Key(char_code as u32));
                        }
                    }
                }
            });
        }

        let mut objects_to_draw = Vec::new(); 
        {
            let mut state = ADAPTER_STATE.lock().unwrap();
            
            for obj in state.objects.iter() {
                if obj.alive && obj.visible {
                    objects_to_draw.push(*obj);
                }
            }

            if state.dirty {
                objects_to_draw.sort_by_key(|o| o.z_index);
                state.dirty = false;
            }
        } 

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                let painter = ui.painter();
                
                for obj in objects_to_draw { 
                    let rect = egui::Rect::from_min_size(
                        egui::pos2(obj.pos.0 as f32, obj.pos.1 as f32),
                        egui::vec2(obj.size.0 as f32, obj.size.1 as f32),
                    );
                    
                    let color = egui::Color32::from_rgba_unmultiplied(
                        obj.color.0, obj.color.1, obj.color.2, obj.color.3,
                    );
                    
                    painter.rect_filled(rect, 0.0, color);
                }
            });
        
        ctx.request_repaint();
    }
}
