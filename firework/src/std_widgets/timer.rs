// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::widget::Widget;

use crate::BuildContext;
use crate::layout::{Constraints, Size};

#[derive(Debug, Clone, Copy)]
pub struct Timer {
    pub time: i32,
    pub delay: i32,
    pub visible: bool,
    pub is_active: bool,
}

impl Timer {
    pub fn new(_layout: u16) -> Option<Self> {
        Some(Self {
            time: 0,
            delay: 0,
            visible: false,
            is_active: false,
        })
    }

    /// Устанавливает Z-индекс
    pub fn z(self, _z: i16) -> Self {
        self
    }

    /// Устанавливает видимость прямоугольника
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn __id(&self) -> usize {
        0
    }

    pub fn __set_vcanvas(self, _handle: usize) -> Self {
        self
    }

    pub fn delay(mut self, delay: i32) -> Self {
        self.time = 0;
        self.delay = delay;
        self.visible = true;
        self.is_active = true;
        self
    }
}

impl Widget for Timer {
    fn position(&self, _position: (i32, i32)) {}

    fn visible(&self, state: bool) {
        Timer::visible(*self, state);
    }

    fn unmount(self) {
        self.visible(false);
        // self.is_active = false;
    }

    fn layout(&mut self, _constraints: Constraints) -> Size {
        Size {
            width: 0,
            height: 0,
        }
    }

    fn get_z_size(&self) -> i16 {
        0
    }

    fn set_z(&self, z: i16) {
        self.z(z);
    }

    fn set_vcanvas(&self, _vcanvas_handle: usize) {}

    fn __event(&mut self, _context: BuildContext) {}
}
