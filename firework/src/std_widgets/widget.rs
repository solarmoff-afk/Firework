// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use crate::layout::{Constraints, Size};

/// Трейт который должны реализовать все скины для поддержки видимости в списках. Он
/// гарантирует наличие метода visible
pub trait Widget {
    fn position(&self, position: (i32, i32));
    fn visible(&self, state: bool);
    fn unmount(self);
    fn layout(&mut self, constraints: Constraints) -> Size;

    /// Сколько z занимает этот виджет (для компоновки)
    fn get_z_size(&self) -> i16;

    /// Устаналивает VCanvas для проекции компонента
    fn set_vcanvas(&self, vcanvas_handle: usize);
}
