// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub mod skins;

mod runtime_errors;

pub use firework_macro::{ui, component};
pub use firework_adapter::{AdapterCommand, AdapterEvent, AdapterClickPhase, AdapterResult};
pub use runtime_errors::RENDER_ADAPTER_MISSING_ERROR;

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
                AdapterEvent::Tick => {
                    adapter_command(AdapterCommand::Render);
                },
                
                _ => {},
            }
        },
    });
}
