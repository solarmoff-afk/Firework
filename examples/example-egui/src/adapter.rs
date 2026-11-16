// [AI GEN ADAPTER EXAMPLE FOR TEST]

use eframe::egui;
use firework_ui::{AdapterClickPhase, AdapterCommand, AdapterEvent, AdapterResult, ShaderLanguage};
use std::sync::Mutex;

const MAX_OBJECTS: usize = 4096;

#[derive(Clone)]
struct RenderObject {
    alive: bool,
    pos: (i32, i32),
    size: (i32, i32),
    color: (u8, u8, u8, u8),
    z_index: i16,
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
    vcanvas: (i32, i32, i32, i32),
    vcanvas_ref: Option<usize>,
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
            vcanvas: (0, 0, 0, 0),
            vcanvas_ref: None,
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
            vcanvas: (0, 0, 0, 0),
            vcanvas_ref: None,
        }
    }
}

struct AdapterState {
    objects: [RenderObject; MAX_OBJECTS],
    listener: Option<fn(AdapterEvent)>,
    dirty: bool,
    // [FIX] Закэшированный z-порядок (индексы живых объектов, отсортированные по z).
    draw_order: Vec<usize>,
    ctx: Option<egui::Context>,
    shader_lang: ShaderLanguage,
    viewport: (u32, u32),

    // [FIX] Pointer capture.
    // Основная кнопка сейчас зажата.
    pointer_down: bool,
    // Идёт обработка Began: ResolveHit запоминает, какой элемент «захватил» указатель.
    capture_armed: bool,
    // hit_group -> элемент, найденный в момент нажатия.
    captures: Vec<(u16, usize)>,
}

static ADAPTER_STATE: Mutex<AdapterState> = Mutex::new(AdapterState {
    objects: [const { RenderObject::empty() }; MAX_OBJECTS],
    listener: None,
    dirty: false,
    draw_order: Vec::new(),
    ctx: None,
    shader_lang: ShaderLanguage::Software,
    viewport: (0, 0),
    pointer_down: false,
    capture_armed: false,
    captures: Vec::new(),
});

fn resolve_vcanvas(objects: &[RenderObject], obj: &RenderObject) -> (i32, i32, i32, i32) {
    if let Some(rect_id) = obj.vcanvas_ref {
        if let Some(target) = objects.get(rect_id) {
            return (target.pos.0, target.pos.1, target.size.0, target.size.1);
        }
    }

    obj.vcanvas
}

/// Обычный hit-test: верхний (по z) видимый объект группы, пересекающий прямоугольник.
fn hit_test(
    objects: &[RenderObject],
    group: u16,
    ax: i32,
    ay: i32,
    aw: i32,
    ah: i32,
) -> Option<usize> {
    let mut highest_z = i16::MIN;
    let mut found_id = None;

    let a_left = ax;
    let a_right = ax + aw;
    let a_top = ay;
    let a_bottom = ay + ah;

    for (id, obj) in objects.iter().enumerate() {
        if obj.alive && obj.visible && obj.hit_group == group {
            let v = resolve_vcanvas(objects, obj);

            let b_left = obj.pos.0 + v.0;
            let b_right = obj.pos.0 + v.0 + obj.size.0;
            let b_top = obj.pos.1 + v.1;
            let b_bottom = obj.pos.1 + v.1 + obj.size.1;

            let intersects =
                a_left < b_right && a_right > b_left && a_top < b_bottom && a_bottom > b_top;

            if intersects && obj.z_index >= highest_z {
                highest_z = obj.z_index;
                found_id = Some(id);
            }
        }
    }

    found_id
}

/// Элемент, захвативший указатель для группы (если он ещё жив и всё ещё в этой группе).
/// Видимость намеренно не проверяем: захват не должен отваливаться посреди перетаскивания.
fn captured_target(state: &AdapterState, group: u16) -> Option<usize> {
    let (_, id) = *state.captures.iter().find(|c| c.0 == group)?;
    let obj = state.objects.get(id)?;

    (obj.alive && obj.hit_group == group).then_some(id)
}

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
                Box::new(|cc| {
                    let shader_lang = if cc.gl.is_some() {
                        ShaderLanguage::Glsl
                    } else {
                        ShaderLanguage::Wgsl
                    };

                    {
                        let mut state = ADAPTER_STATE.lock().unwrap();
                        state.shader_lang = shader_lang;
                    }

                    Ok(Box::new(FireworkEguiApp::default()))
                }),
            );

            AdapterResult::Void
        }

        AdapterCommand::RemoveAll => {
            for obj in state.objects.iter_mut() {
                obj.alive = false;
                obj.text_segments.clear();
                obj.clip_to = None;
                obj.vcanvas_ref = None;
            }

            state.captures.clear();
            state.dirty = true;

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
            // Проверка на то, что окно открыто. В таком случае можно измерить текст
            if let Some(ctx) = &state.ctx {
                if let Some(obj) = state.objects.get(id) {
                    if obj.alive && obj.is_text {
                        let job = create_layout_job(obj);
                        let galley = ctx.fonts(|f| f.layout_job(job));

                        return AdapterResult::Size(
                            galley.rect.width().ceil() as u32,
                            galley.rect.height().ceil() as u32,
                        );
                    }
                }
            } else {
                // Если окно ещё не открыто то примерный размер
                if let Some(obj) = state.objects.get(id) {
                    if obj.alive && obj.is_text {
                        let text_len = obj
                            .text_segments
                            .iter()
                            .map(|(s, _)| s.chars().count())
                            .sum::<usize>();
                        // Примерная ширина это количество букв * (размер шрифта / 2)
                        let w = text_len as u32 * (obj.font_size as u32 / 2);

                        // Примерная высота это размер шрифта + 20%
                        let h = (obj.font_size as f32 * 1.2) as u32;

                        return AdapterResult::Size(w, h);
                    }
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

        AdapterCommand::SetVCanvas(id, rect) => {
            if let Some(obj) = state.objects.get_mut(id) {
                obj.vcanvas = rect;
                state.dirty = true;
            }

            AdapterResult::Void
        }

        AdapterCommand::SharedVCanvas(id, rect_id) => {
            if let Some(obj) = state.objects.get_mut(id) {
                obj.vcanvas_ref = Some(rect_id);
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
                obj.vcanvas_ref = None;
                state.dirty = true;
            }

            // [FIX] Слот может быть переиспользован через NewRect — захват на него снимаем.
            state.captures.retain(|c| c.1 != id);

            AdapterResult::Void
        }

        AdapterCommand::SetHitGroup(id, group) => {
            if let Some(obj) = state.objects.get_mut(id) {
                obj.hit_group = group;
            }

            AdapterResult::Void
        }

        AdapterCommand::ResolveHit(group, (ax, ay, aw, ah)) => {
            // [FIX] Pointer capture.
            // Пока кнопка зажата, группа «прилипает» к элементу, найденному в момент
            // нажатия, независимо от того, где сейчас курсор. Иначе при быстром движении
            // курсор вылетает за границы элемента (тот отстаёт на кадр), hit-test
            // промахивается и перетаскивание обрывается.
            if state.pointer_down && !state.capture_armed {
                if let Some(id) = captured_target(&state, group) {
                    return AdapterResult::Handle(id);
                }
            }

            let found = hit_test(&state.objects, group, ax, ay, aw, ah);

            // Момент нажатия (обработчик Began): запоминаем захватчика для этой группы.
            if state.capture_armed {
                if let Some(id) = found {
                    if !state.captures.iter().any(|c| c.0 == group) {
                        state.captures.push((group, id));
                    }
                }
            }

            match found {
                Some(id) => AdapterResult::Handle(id),
                None => AdapterResult::Fail,
            }
        }

        AdapterCommand::GetShaderLanguage => AdapterResult::ShaderLang(state.shader_lang),

        AdapterCommand::GetViewport => {
            let size = if let Some(ctx) = &state.ctx {
                let rect = ctx.input(|i| i.screen_rect());
                let w = rect.width().max(0.0) as u32;
                let h = rect.height().max(0.0) as u32;
                state.viewport = (w, h);
                (w, h)
            } else {
                state.viewport
            };

            AdapterResult::Size(size.0, size.1)
        }

        AdapterCommand::Render => AdapterResult::Void,
    }
}

/// Входные события, собранные за кадр. Собираем их заранее, вне `ctx.input(..)`,
/// чтобы listener не вызывался под блокировкой контекста egui
/// (иначе вызов `MeasureText` из обработчика может заблокироваться).
enum PointerInput {
    Press(u32, u32),
    Move(u32, u32),
    Release(u32, u32),
    Key(u32),
}

fn to_px(pos: egui::Pos2) -> (u32, u32) {
    (pos.x.max(0.0) as u32, pos.y.max(0.0) as u32)
}

fn dispatch_input(listener: fn(AdapterEvent), input: PointerInput) {
    match input {
        PointerInput::Press(x, y) => {
            {
                // Включаем захват на время обработчика Began: ResolveHit внутри него
                // запомнит, какой элемент оказался под курсором.
                let mut state = ADAPTER_STATE.lock().unwrap();
                state.pointer_down = true;
                state.capture_armed = true;
                state.captures.clear();
            }

            listener(AdapterEvent::Touch(x, y, AdapterClickPhase::Began));

            ADAPTER_STATE.lock().unwrap().capture_armed = false;
        }

        PointerInput::Move(x, y) => {
            listener(AdapterEvent::Touch(x, y, AdapterClickPhase::Moved));
        }

        PointerInput::Release(x, y) => {
            // Ended тоже уходит владельцу захвата, поэтому захват снимаем только после него.
            listener(AdapterEvent::Touch(x, y, AdapterClickPhase::Ended));

            let mut state = ADAPTER_STATE.lock().unwrap();
            state.pointer_down = false;
            state.capture_armed = false;
            state.captures.clear();
        }

        PointerInput::Key(code) => {
            listener(AdapterEvent::Key(code));
        }
    }
}

#[derive(Default)]
struct FireworkEguiApp {
    // Основная кнопка зажата (с точки зрения адаптера).
    pointer_down: bool,
    // Последняя известная позиция курсора (egui при выходе из окна отдаёт None).
    last_pos: egui::Pos2,
}

impl FireworkEguiApp {
    /// Превращает сырые события egui в упорядоченный список событий для listener.
    ///
    /// Вместо «одно событие на кадр по latest_pos» обрабатываем каждое событие мыши:
    ///  - Began получает точную позицию нажатия, а не позицию в конце кадра;
    ///  - нажатие и отпускание в одном кадре не теряются;
    ///  - выход курсора из окна (PointerGone) с зажатой кнопкой не обрывает драг;
    ///  - Ended гарантированно приходит, даже если egui потерял событие отпускания.
    fn collect_input(&mut self, ctx: &egui::Context) -> Vec<PointerInput> {
        let mut out = Vec::new();

        ctx.input(|i| {
            // Движения внутри кадра склеиваем в одно (хватает последней позиции),
            // но порядок относительно нажатия/отпускания сохраняем.
            let mut pending_move: Option<egui::Pos2> = None;

            for event in &i.events {
                match event {
                    egui::Event::PointerMoved(pos) => {
                        self.last_pos = *pos;

                        if self.pointer_down {
                            pending_move = Some(*pos);
                        }
                    }

                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        ..
                    } => {
                        if let Some(p) = pending_move.take() {
                            let (x, y) = to_px(p);
                            out.push(PointerInput::Move(x, y));
                        }

                        self.last_pos = *pos;
                        let (x, y) = to_px(*pos);

                        if *pressed {
                            if !self.pointer_down {
                                self.pointer_down = true;
                                out.push(PointerInput::Press(x, y));
                            }
                        } else if self.pointer_down {
                            self.pointer_down = false;
                            out.push(PointerInput::Release(x, y));
                        }
                    }

                    egui::Event::Text(text) => {
                        for c in text.chars() {
                            out.push(PointerInput::Key(c as u32));
                        }
                    }

                    egui::Event::Key {
                        key: egui::Key::Backspace,
                        pressed: true,
                        ..
                    } => {
                        out.push(PointerInput::Key(8));
                    }

                    egui::Event::Key {
                        key: egui::Key::Enter,
                        pressed: true,
                        ..
                    } => {
                        out.push(PointerInput::Key(13));
                    }

                    // PointerGone (курсор вышел из окна) намеренно игнорируем:
                    // last_pos и pointer_down сохраняются, драг продолжается.
                    _ => {}
                }
            }

            if let Some(p) = pending_move.take() {
                if self.pointer_down {
                    let (x, y) = to_px(p);
                    out.push(PointerInput::Move(x, y));
                }
            }

            // Страховка от «залипшего» драга: кнопка уже не зажата, а Release мы не видели
            // (отпустили за окном, потеря фокуса и т.п.).
            if self.pointer_down && !i.pointer.primary_down() {
                self.pointer_down = false;

                let (x, y) = to_px(self.last_pos);
                out.push(PointerInput::Release(x, y));
            }
        });

        out
    }
}

impl eframe::App for FireworkEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let (listener, viewport_event) = {
            let mut state = ADAPTER_STATE.lock().unwrap();
            state.ctx = Some(ctx.clone());

            let rect = ctx.input(|i| i.screen_rect());
            let w = rect.width().max(0.0) as u32;
            let h = rect.height().max(0.0) as u32;
            let old = state.viewport;
            state.viewport = (w, h);

            let changed = old != (0, 0) && old != (w, h);

            (state.listener, if changed { Some((w, h)) } else { None })
        };

        let inputs = self.collect_input(ctx);

        if let Some(listener) = listener {
            if let Some((w, h)) = viewport_event {
                listener(AdapterEvent::Viewport(w, h));
            }

            let dt_seconds = ctx.input(|i| i.stable_dt);
            let dt_ms = (dt_seconds * 1000.0).round() as u32;
            let safe_dt_ms = dt_ms.min(250);

            listener(AdapterEvent::Tick(safe_dt_ms));

            if ctx.input(|i| i.viewport().close_requested()) {
                listener(AdapterEvent::CloseRequest);
            }

            for input in inputs {
                dispatch_input(listener, input);
            }
        }

        let mut objects_to_draw = Vec::new();
        let mut clip_rects = std::collections::HashMap::new();
        {
            let mut state = ADAPTER_STATE.lock().unwrap();
            let st = &mut *state;

            // [FIX] Раньше сортировка применялась к списку, который каждый кадр строился
            // заново, поэтому z-порядок соблюдался только в «грязных» кадрах.
            // Теперь порядок кэшируется в draw_order и пересобирается только при dirty.
            // Сортировка стабильная: при равном z порядок по индексу, как и в hit_test.
            if st.dirty {
                let mut order: Vec<usize> =
                    (0..MAX_OBJECTS).filter(|&i| st.objects[i].alive).collect();
                order.sort_by_key(|&i| st.objects[i].z_index);

                st.draw_order = order;
                st.dirty = false;
            }

            for &i in &st.draw_order {
                let obj = &st.objects[i];

                if !obj.alive {
                    continue;
                }

                let v = resolve_vcanvas(&st.objects, obj);

                if obj.visible {
                    let mut draw_obj = obj.clone();
                    draw_obj.vcanvas = v;
                    objects_to_draw.push(draw_obj);
                }

                clip_rects.insert(
                    i,
                    egui::Rect::from_min_size(
                        egui::pos2((obj.pos.0 + v.0) as f32, (obj.pos.1 + v.1) as f32),
                        egui::vec2(obj.size.0 as f32, obj.size.1 as f32),
                    ),
                );
            }
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                for obj in objects_to_draw {
                    let rect = egui::Rect::from_min_size(
                        egui::pos2(
                            (obj.pos.0 + obj.vcanvas.0) as f32,
                            (obj.pos.1 + obj.vcanvas.1) as f32,
                        ),
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

                    let current_painter = if obj.vcanvas.2 > 0 && obj.vcanvas.3 > 0 {
                        let vcanvas_rect = egui::Rect::from_min_size(
                            egui::pos2(obj.vcanvas.0 as f32, obj.vcanvas.1 as f32),
                            egui::vec2(obj.vcanvas.2 as f32, obj.vcanvas.3 as f32),
                        );

                        current_painter.with_clip_rect(vcanvas_rect)
                    } else {
                        current_painter
                    };

                    if obj.is_text {
                        let job = create_layout_job(&obj);
                        let pos = egui::pos2(
                            (obj.pos.0 + obj.vcanvas.0) as f32,
                            (obj.pos.1 + obj.vcanvas.1) as f32,
                        );
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
