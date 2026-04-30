// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub mod skins;
pub mod dyn_list;
pub mod null_adapter;

mod runtime_errors;

pub use firework_macro::{ui, shared, component, effect};
pub use firework_adapter::{AdapterCommand, AdapterEvent, AdapterClickPhase, AdapterResult};

pub use runtime_errors::RENDER_ADAPTER_MISSING_ERROR;
pub use skins::DefaultRectSkin;
pub use dyn_list::{DynList, ListEntry};
pub use null_adapter::null_adapter;

pub const TOUCH_HIT_GROUP: u16 = u16::MAX;

/// Type for component props
pub type Prop<T> = Option<T>;

pub struct ComponentContext {
    pub depth: u16,
}

impl ComponentContext {
    pub fn new() -> Self {
        Self {
            depth: 0,
        }
    }
}

/// A simpler implementation of the matches macro separate from STD for use in
/// generated code
#[macro_export]
macro_rules! tiny_matches {
    ($expression:expr, $($pattern:pat_param)|+ $(if $guard:expr)?) => {
        match $expression {
            $($pattern)|+ $(if $guard)? => true,
            _ => false
        }
    };
}

#[derive(Debug, Clone, Copy)]
pub enum CurrentEvent {
    None,
    Touch {
        x: u32,
        y: u32,
        hit_object_id: Option<usize>,
        phase: AdapterClickPhase,
    },
}

#[cfg(not(feature = "safety-multithread"))]
static mut CURRENT_EVENT: CurrentEvent = CurrentEvent::None;

#[cfg(feature = "safety-multithread")]
static CURRENT_EVENT: OnceLock<Mutex<CurrentEvent>> = OnceLock::new();

/// Установить текущее событие
#[cfg(not(feature = "safety-multithread"))]
pub fn set_current_event(event: CurrentEvent) {
    unsafe {
        CURRENT_EVENT = event;
    }
}

#[cfg(feature = "safety-multithread")]
pub fn set_current_event(event: CurrentEvent) {
    *CURRENT_EVENT.get_or_init(|| Mutex::new(CurrentEvent::None)).lock().unwrap() = event;
}

/// Получить и ОЧИСТИТЬ текущее событие (заменить на None)
#[cfg(not(feature = "safety-multithread"))]
pub fn take_current_event() -> CurrentEvent {
    unsafe {
        let event = CURRENT_EVENT;
        CURRENT_EVENT = CurrentEvent::None;
        event
    }
}

#[cfg(feature = "safety-multithread")]
pub fn take_current_event() -> CurrentEvent {
    let mut lock = CURRENT_EVENT.get_or_init(|| Mutex::new(CurrentEvent::None)).lock().unwrap();
    let event = *lock;
    *lock = CurrentEvent::None;
    event
}

/// Current Flash pass context of the screen or component
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LifeCycle {
    Zero,
    Event,
    Build,
    Navigate,
    Reactive,
}

#[cfg(not(feature = "safety-multithread"))]
// Хранилище текущего фокуса (активного слайда) для ивентов
static mut CURRENT_FOCUS: Option<fn()> = None;

#[cfg(not(feature = "safety-multithread"))]
static mut CURRENT_ADAPTER: Option<fn(AdapterCommand) -> AdapterResult> = None;

#[cfg(not(feature = "safety-multithread"))]
// Айди текущего фокуса для сравнения
static mut CURRENT_FOCUS_ID: Option<u128> = None;

#[cfg(feature = "safety-multithread")]
use std::sync::{Mutex, OnceLock};

#[cfg(feature = "safety-multithread")]
static CURRENT_FOCUS: OnceLock<Mutex<Option<fn()>>> = OnceLock::new();

#[cfg(feature = "safety-multithread")]
static CURRENT_ADAPTER: OnceLock<Mutex<Option<fn(AdapterCommand) -> AdapterResult>>> = OnceLock::new();

#[cfg(feature = "safety-multithread")]
static CURRENT_FOCUS_ID: OnceLock<Mutex<Option<u128>>> = OnceLock::new();

/// Возвращает указатель на текущий активный слайд. Используется макросом для
/// определения явлется ли лайфцикл Build
#[cfg(not(feature = "safety-multithread"))]
pub fn get_focus() -> fn() {
    unsafe { CURRENT_FOCUS.unwrap_or(|| {}) }
}

#[cfg(feature = "safety-multithread")]
pub fn get_focus() -> fn() {
    CURRENT_FOCUS.get_or_init(|| Mutex::new(None)).lock().unwrap().unwrap_or(|| {})
}

/// Устанавливает текущий слайд в глобальный фокус
#[cfg(not(feature = "safety-multithread"))]
pub fn set_focus(f: fn()) {
    unsafe {
        CURRENT_FOCUS = Some(f);
    }
}

#[cfg(not(feature = "safety-multithread"))]
pub fn set_adapter(f: fn(AdapterCommand) -> AdapterResult) {
    unsafe {
        CURRENT_ADAPTER = Some(f);
    }
}

#[cfg(feature = "safety-multithread")]
pub fn set_adapter(f: fn(AdapterCommand) -> AdapterResult) {
    CURRENT_ADAPTER.get_or_init(|| Mutex::new(Some(f))).lock().unwrap().expect(RENDER_ADAPTER_MISSING_ERROR);
}

#[cfg(feature = "safety-multithread")]
pub fn get_adapter() -> fn(AdapterCommand) -> AdapterResult {
    CURRENT_ADAPTER.get_or_init(|| Mutex::new(None)).lock().unwrap().expect(RENDER_ADAPTER_MISSING_ERROR)
}

#[cfg(not(feature = "safety-multithread"))]
pub fn get_adapter() -> fn(AdapterCommand) -> AdapterResult {
    unsafe { CURRENT_ADAPTER.expect(RENDER_ADAPTER_MISSING_ERROR) }
}

#[cfg(feature = "safety-multithread")]
pub fn set_focus(f: fn()) {
    *CURRENT_FOCUS.get_or_init(|| Mutex::new(None)).lock().unwrap() = Some(f);
}

#[cfg(not(feature = "safety-multithread"))]
pub fn get_focus_id() -> u128 {
    unsafe { CURRENT_FOCUS_ID.unwrap_or(0) }
}

#[cfg(feature = "safety-multithread")]
pub fn get_focus_id() -> u128 {
    CURRENT_FOCUS_ID.get_or_init(|| Mutex::new(None)).lock().unwrap().unwrap_or(0)
}

#[cfg(not(feature = "safety-multithread"))]
pub fn set_focus_id(id: u128) {
    unsafe {
        CURRENT_FOCUS_ID = Some(id);
    }
}

#[cfg(feature = "safety-multithread")]
pub fn set_focus_id(id: u128) {
    *CURRENT_FOCUS_ID.get_or_init(|| Mutex::new(None)).lock().unwrap() = Some(id);
}

pub fn adapter_command(command: AdapterCommand) -> AdapterResult {
    get_adapter()(command)
}

fn default_adapter(command: AdapterCommand) -> AdapterResult {
    match command {
        _ => {},
    }

    AdapterResult::Void
}

pub fn run(root_slide: fn()) {
    set_adapter(default_adapter);
    root_slide();

    after_first_flash();
}

pub fn run_with_adapter(adapter: fn(AdapterCommand) -> AdapterResult, root_slide: fn()) {
    set_adapter(adapter);
    root_slide();

    after_first_flash();
}

pub fn after_first_flash() {
    adapter_command(AdapterCommand::RunLoop {
        title: "Test",
        width: 720,
        height: 1280,
        listener: |event| {
            match event {
                AdapterEvent::Touch(x, y, phase) => {
                    handle_touch_event(x, y, phase, TOUCH_HIT_GROUP);
                },

                AdapterEvent::Tick => {
                    adapter_command(AdapterCommand::Render);
                },
                
                _ => {},
            }
        },
    });
}

pub fn handle_touch_event(x: u32, y: u32, phase: AdapterClickPhase, hit_group: u16) {
    let hit_result = adapter_command(AdapterCommand::ResolveHit(hit_group, (x as i32, y as i32, 1, 1)));
    
    let hit_object_id = match hit_result {
        AdapterResult::Handle(id) => Some(id),
        _ => None,
    };
    
    dispatch_event(CurrentEvent::Touch {
        x,
        y,
        hit_object_id,
        phase,
    });
}

pub fn dispatch_event(event: CurrentEvent) {
    set_current_event(event);
    get_focus()();
    set_current_event(CurrentEvent::None);
}
