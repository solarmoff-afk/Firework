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

// Хранилище текущего фокуса (активного слайда) для ивентов
static mut CURRENT_FOCUS: Option<fn()> = None;

// Айди текущего фокуса для сравнения
static mut CURRENT_FOCUS_ID: Option<u128> = None;

/// Возвращает указатель на текущий активный слайд. Используется макросом для
/// определения явлется ли лайфцикл Build
pub fn get_focus() -> fn() {
    unsafe { CURRENT_FOCUS.unwrap_or(|| {}) }
}

/// Устанавливает текущий слайд в глобальный фокус
pub fn set_focus(f: fn()) {
    unsafe {
        CURRENT_FOCUS = Some(f);
    }
}

pub fn get_focus_id() -> u128 {
    unsafe { CURRENT_FOCUS_ID.unwrap_or(0) }
}

pub fn set_focus_id(id: u128) {
    unsafe {
        CURRENT_FOCUS_ID = Some(id);
    }
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
