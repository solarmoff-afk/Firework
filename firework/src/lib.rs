// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub use firework_macro::{ui, component};

/// Состояния жизненного цикла
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LifeCycle {
    Zero,
    Event,
    Build,
    Navigate,
}

/// Команды для низкоуровневого графического адаптера
pub enum AdapterCommand {
    RemoveAll,
}

#[cfg(not(feature = "safety-multithread"))]
// Хранилище текущего фокуса (активного слайда) для ивентов
static mut CURRENT_FOCUS: Option<fn()> = None;

#[cfg(not(feature = "safety-multithread"))]
// Айди текущего фокуса для сравнения
static mut CURRENT_FOCUS_ID: Option<u128> = None;

#[cfg(feature = "safety-multithread")]
use std::sync::{Mutex, OnceLock};

#[cfg(feature = "safety-multithread")]
static CURRENT_FOCUS: OnceLock<Mutex<Option<fn()>>> = OnceLock::new();

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

/// Точка входа для адаптера отрисовки. 
/// Макрос генерирует вызовы этой функции для управления кадрами.
pub fn adapter_command(command: AdapterCommand) {
    match command {
        AdapterCommand::RemoveAll => {
            // TODO
        }
    }
}

pub fn run(root_slide: fn()) {
    root_slide();
}
