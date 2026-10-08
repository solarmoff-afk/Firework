// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use crate::BuildContext;
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

    fn set_z(&self, z: i16);

    /// Устаналивает VCanvas для проекции компонента
    fn set_vcanvas(&self, vcanvas_handle: usize);

    /// Для стандартных виджетов заглушка, для компонентов передача
    fn __event(&mut self, context: BuildContext);
}

/// Реализация Widget для &mut T, так как он нужен в safety режиме на этапе расчёта Z. &mut
/// здесь нужен для того, чтобы не было ошибки из-за метода layout
impl<T: Widget + ?Sized> Widget for &mut T {
    fn position(&self, position: (i32, i32)) {
        (**self).position(position);
    }

    fn visible(&self, state: bool) {
        (**self).visible(state);
    }

    // Паники не будет, так как для расчёта z не нужен unmount
    fn unmount(self) {
        panic!("Cannot unmount a reference");
    }

    fn layout(&mut self, constraints: Constraints) -> Size {
        (**self).layout(constraints)
    }

    fn get_z_size(&self) -> i16 {
        (**self).get_z_size()
    }

    fn set_z(&self, z: i16) {
        (**self).set_z(z);
    }

    fn set_vcanvas(&self, vcanvas_handle: usize) {
        (**self).set_vcanvas(vcanvas_handle);
    }

    fn __event(&mut self, context: BuildContext) {
        (**self).__event(context);
    }
}
