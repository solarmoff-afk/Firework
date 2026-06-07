// [AI GEN ADAPTER EXAMPLE FOR TEST]

use eframe::egui;
use firework_ui::{AdapterClickPhase, AdapterCommand, AdapterEvent, AdapterResult};
use std::sync::Mutex;

const MAX_OBJECTS: usize = 4096;

#[derive(Clone)]
struct RenderObject {
    alive: bool,
    pos: (i32, i32),
    size: (i32, i32),
    color: (u8, u8, u8, u8),
    z_index: i32,
    visible: bool,
    hit_group: u16,
    is_text: bool,
    text_segments: Vec<(String, u8)>,
    text_align: u8,
    text_wrap_width: u32,
    corner_radius: (u16, u16, u16, u16),
    border_width: u16,
    border_color: (u8, u8, u8, u8),
    font_size: u16,
    clip_to: Option<usize>,
}

impl Default for RenderObject {
    fn default() -> Self {
        Self {
            alive: false,
            pos: (0, 0),
            size: (0, 0),
            color: (255, 255, 255, 255),
            z_index: 0,
            visible: true,
            hit_group: 0,
            is_text: false,
            text_segments: Vec::new(),
            text_align: 0,
            text_wrap_width: 0,
            corner_radius: (0, 0, 0, 0),
            border_width: 0,
            border_color: (0, 0, 0, 0),
            font_size: 14,
            clip_to: None,
        }
    }
}

impl RenderObject {
    const fn empty() -> Self {
        Self {
            alive: false,
            pos: (0, 0),
            size: (0, 0),
            color: (0, 0, 0, 0),
            z_index: 0,
            visible: false,
            hit_group: 0,
            is_text: false,
            text_segments: Vec::new(),
            text_align: 0,
            text_wrap_width: 0,
            corner_radius: (0, 0, 0, 0),
            border_width: 0,
            border_color: (0, 0, 0, 0),
            font_size: 14,
            clip_to: None,
        }
    }
}

struct AdapterState {
    objects: [RenderObject; MAX_OBJECTS],
    listener: Option<fn(AdapterEvent)>,
    dirty: bool,
    ctx: Option<egui::Context>,
}

static ADAPTER_STATE: Mutex<AdapterState> = Mutex::new(AdapterState {
    objects: [const { RenderObject::empty() }; MAX_OBJECTS],
    listener: None,
    dirty: false,
    ctx: None,
});

fn create_layout_job(obj: &RenderObject) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();

    job.halign = match obj.text_align {
        1 => egui::Align::Center,
        2 => egui::Align::Max,
        _ => egui::Align::Min,
    };

    if obj.text_wrap_width > 0 {
        job.wrap.max_width = obj.text_wrap_width as f32;
        job.wrap.break_anywhere = false;
    } else {
        job.wrap.max_width = f32::INFINITY;
        job.wrap.break_anywhere = false;
    }

    let color =
        egui::Color32::from_rgba_unmultiplied(obj.color.0, obj.color.1, obj.color.2, obj.color.3);

    let font_size = obj.font_size as f32;

    for (text, mode) in &obj.text_segments {
        let mut format = egui::text::TextFormat {
            font_id: egui::FontId::proportional(font_size),
            color,
            ..Default::default()
        };

        match mode {
            1 => {
                format.font_id =
                    egui::FontId::new(font_size, egui::FontFamily::Name("Bold".into()));
            }
            2 => {
                format.italics = true;
            }
            3 => {
                format.font_id =
                    egui::FontId::new(font_size, egui::FontFamily::Name("Bold".into()));
                format.italics = true;
            }
            _ => {}
        }

        job.append(text, 0.0, format);
    }

    job
}

pub fn egui_adapter(cmd: AdapterCommand<'_>) -> AdapterResult {
    let mut state = ADAPTER_STATE.lock().unwrap();

    match cmd {
        AdapterCommand::RunLoop {
            title,
            width,
            height,
            listener,
        } => {
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
        }

        AdapterCommand::RemoveAll => {
            for obj in state.objects.iter_mut() {
                obj.alive = false;
                obj.text_segments.clear();
                obj.clip_to = None;
            }

            AdapterResult::Void
        }

        AdapterCommand::NewRect { layout: _ } => {
            if let Some(index) = state.objects.iter().position(|obj| !obj.alive) {
                state.objects[index] = RenderObject {
                    alive: true,
                    visible: true,
                    is_text: false,
                    ..Default::default()
                };
                state.dirty = true;

                AdapterResult::Handle(index)
            } else {
                AdapterResult::Fail
            }
        }

        AdapterCommand::NewText { layout: _ } => {
            if let Some(index) = state.objects.iter().position(|obj| !obj.alive) {
                state.objects[index] = RenderObject {
                    alive: true,
                    visible: true,
                    is_text: true,
                    text_segments: Vec::new(),
                    ..Default::default()
                };
                state.dirty = true;

                AdapterResult::Handle(index)
            } else {
                AdapterResult::Fail
            }
        }

        AdapterCommand::PushText { handle, text, mode } => {
            if let Some(obj) = state.objects.get_mut(handle) {
                obj.text_segments.push((text.to_string(), mode));
                state.dirty = true;
            }

            AdapterResult::Void
        }

        AdapterCommand::ClearText(id) => {
            if let Some(obj) = state.objects.get_mut(id) {
                obj.text_segments.clear();
                state.dirty = true;
            }

            AdapterResult::Void
        }

        AdapterCommand::MeasureText(id) => {
            if state.ctx.is_none() {
                state.ctx = Some(egui::Context::default());
            }

            if let Some(obj) = state.objects.get(id) {
                if obj.alive && obj.is_text {
                    let job = create_layout_job(obj);
                    let galley = state.ctx.as_ref().unwrap().fonts(|f| f.layout_job(job));
                    return AdapterResult::Size(
                        galley.rect.width().ceil() as u32,
                        galley.rect.height().ceil() as u32,
                    );
                }
            }
            AdapterResult::Size(0, 0)
        }

        AdapterCommand::SetTextAlign(id, align) => {
            if let Some(obj) = state.objects.get_mut(id) {
                obj.text_align = align;
                state.dirty = true;
            }

            AdapterResult::Void
        }

        AdapterCommand::SetTextWrapWidth(id, width) => {
            if let Some(obj) = state.objects.get_mut(id) {
                obj.text_wrap_width = width;
                state.dirty = true;
            }

            AdapterResult::Void
        }

        AdapterCommand::SetPosition(id, pos) => {
            if let Some(obj) = state.objects.get_mut(id) {
                obj.pos = pos;
            }

            AdapterResult::Void
        }

        AdapterCommand::SetSize(id, size) => {
            if let Some(obj) = state.objects.get_mut(id) {
                obj.size = size;
            }

            AdapterResult::Void
        }

        AdapterCommand::SetColor(id, color) => {
            if let Some(obj) = state.objects.get_mut(id) {
                obj.color = color;
            }

            AdapterResult::Void
        }

        AdapterCommand::SetZ(id, z) => {
            if let Some(obj) = state.objects.get_mut(id) {
                obj.z_index = z;
                state.dirty = true;
            }

            AdapterResult::Void
        }

        AdapterCommand::SetVisible(id, visible) => {
            if let Some(obj) = state.objects.get_mut(id) {
                obj.visible = visible;
            }

            AdapterResult::Void
        }

        AdapterCommand::SetCornerRadius(id, radius) => {
            if let Some(obj) = state.objects.get_mut(id) {
                obj.corner_radius = radius;
                state.dirty = true;
            }

            AdapterResult::Void
        }

        AdapterCommand::SetBorder(id, width, color) => {
            if let Some(obj) = state.objects.get_mut(id) {
                obj.border_width = width;
                obj.border_color = color;
                state.dirty = true;
            }

            AdapterResult::Void
        }

        AdapterCommand::SetFontSize(id, size) => {
            if let Some(obj) = state.objects.get_mut(id) {
                obj.font_size = size;
                state.dirty = true;
            }

            AdapterResult::Void
        }

        AdapterCommand::SetClipTo(id, clip_id) => {
            if let Some(obj) = state.objects.get_mut(id) {
                obj.clip_to = Some(clip_id);
                state.dirty = true;
            }

            AdapterResult::Void
        }

        AdapterCommand::SetShadow(..) => AdapterResult::Void,

        AdapterCommand::Remove(id) => {
            if let Some(obj) = state.objects.get_mut(id) {
                obj.alive = false;
                obj.text_segments.clear();
                obj.clip_to = None;
                state.dirty = true;
            }

            AdapterResult::Void
        }

        AdapterCommand::SetHitGroup(id, group) => {
            if let Some(obj) = state.objects.get_mut(id) {
                obj.hit_group = group;
            }

            AdapterResult::Void
        }

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

                    let intersects = a_left < b_right
                        && a_right > b_left
                        && a_top < b_bottom
                        && a_bottom > b_top;

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
        }

        AdapterCommand::Render => AdapterResult::Void,
    }
}

struct FireworkEguiApp;

impl eframe::App for FireworkEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let listener = {
            let mut state = ADAPTER_STATE.lock().unwrap();
            state.ctx = Some(ctx.clone());
            state.listener
        };

        if let Some(listener) = listener {
            listener(AdapterEvent::Tick);

            if ctx.input(|i| i.viewport().close_requested()) {
                listener(AdapterEvent::CloseRequest);
            }

            ctx.input(|i| {
                if let Some(pos) = i.pointer.latest_pos() {
                    let x = pos.x.max(0.0) as u32;
                    let y = pos.y.max(0.0) as u32;

                    if i.pointer.any_pressed() {
                        listener(AdapterEvent::Touch(x, y, AdapterClickPhase::Began));
                    } else if i.pointer.any_down() && i.pointer.delta() != egui::Vec2::ZERO {
                        listener(AdapterEvent::Touch(x, y, AdapterClickPhase::Moved));
                    } else if i.pointer.any_released() {
                        listener(AdapterEvent::Touch(x, y, AdapterClickPhase::Ended));
                    }
                }

                for event in &i.events {
                    match event {
                        egui::Event::Text(text) => {
                            for c in text.chars() {
                                listener(AdapterEvent::Key(c as u32));
                            }
                        }
                        egui::Event::Key {
                            key: egui::Key::Backspace,
                            pressed: true,
                            ..
                        } => {
                            listener(AdapterEvent::Key(8));
                        }
                        egui::Event::Key {
                            key: egui::Key::Enter,
                            pressed: true,
                            ..
                        } => {
                            listener(AdapterEvent::Key(13));
                        }
                        _ => {}
                    }
                }
            });
        }

        let mut objects_to_draw = Vec::new();
        let mut clip_rects = std::collections::HashMap::new();
        {
            let mut state = ADAPTER_STATE.lock().unwrap();

            for (i, obj) in state.objects.iter().enumerate() {
                if obj.alive {
                    if obj.visible {
                        objects_to_draw.push(obj.clone());
                    }

                    clip_rects.insert(
                        i,
                        egui::Rect::from_min_size(
                            egui::pos2(obj.pos.0 as f32, obj.pos.1 as f32),
                            egui::vec2(obj.size.0 as f32, obj.size.1 as f32),
                        ),
                    );
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
                for obj in objects_to_draw {
                    let rect = egui::Rect::from_min_size(
                        egui::pos2(obj.pos.0 as f32, obj.pos.1 as f32),
                        egui::vec2(obj.size.0 as f32, obj.size.1 as f32),
                    );

                    let color = egui::Color32::from_rgba_unmultiplied(
                        obj.color.0,
                        obj.color.1,
                        obj.color.2,
                        obj.color.3,
                    );

                    let current_painter = if let Some(clip_id) = obj.clip_to {
                        if let Some(clip_rect) = clip_rects.get(&clip_id) {
                            ui.painter().with_clip_rect(*clip_rect)
                        } else {
                            ui.painter().clone()
                        }
                    } else {
                        ui.painter().clone()
                    };

                    if obj.is_text {
                        let job = create_layout_job(&obj);
                        let pos = egui::pos2(obj.pos.0 as f32, obj.pos.1 as f32);
                        current_painter.galley(pos, ctx.fonts(|f| f.layout_job(job)), color);
                    } else {
                        let rounding = egui::Rounding {
                            nw: obj.corner_radius.0 as f32,
                            ne: obj.corner_radius.1 as f32,
                            se: obj.corner_radius.2 as f32,
                            sw: obj.corner_radius.3 as f32,
                        };

                        let stroke_color = egui::Color32::from_rgba_unmultiplied(
                            obj.border_color.0,
                            obj.border_color.1,
                            obj.border_color.2,
                            obj.border_color.3,
                        );
                        let stroke = egui::Stroke::new(obj.border_width as f32, stroke_color);

                        current_painter.rect(rect, rounding, color, stroke);
                    }
                }
            });

        ctx.request_repaint();
    }
}
