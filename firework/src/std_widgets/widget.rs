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
}
