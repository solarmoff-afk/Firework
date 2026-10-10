// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::widget::Widget;

use crate::BuildContext;
use crate::layout::{Constraints, Size};

use core::cell::Cell;

#[derive(Debug)]
pub struct Timer<T> {
    pub time: Cell<i32>,
    pub delay: Cell<i32>,
    visible: Cell<bool>,
    pub is_active: Cell<bool>,
    pub output_value: Option<T>,
}

impl<T> Timer<T> {
    pub fn new(_layout: u16) -> Option<Self> {
        Some(Self {
            time: Cell::new(0),
            delay: Cell::new(0),
            visible: Cell::new(false),
            is_active: Cell::new(false),
            output_value: None,
        })
    }

    /// Устанавливает Z-индекс
    pub fn z(self, _z: i16) -> Self {
        self
    }

    /// Устанавливает видимость прямоугольника
    pub fn visible(self, visible: bool) -> Self {
        self.visible.set(visible);
        self
    }

    /// Возвращает текущую видимость
    pub fn __visible(&self) -> bool {
        self.visible.get()
    }

    pub fn __id(&self) -> usize {
        0
    }

    pub fn __set_vcanvas(self, _handle: usize) -> Self {
        self
    }

    pub fn delay(self, delay: i32) -> Self {
        self.time.set(0);
        self.delay.set(delay);
        self.visible.set(true);
        self.is_active.set(true);
        self
    }

    pub fn output(mut self, value: T) -> Self {
        self.output_value = Some(value);
        self
    }
}

impl<T> Widget for Timer<T> {
    fn position(&self, _position: (i32, i32)) {}

    fn visible(&self, state: bool) {
        self.visible.set(state);
    }

    fn unmount(self) {
        self.visible.set(false);
        // self.is_active.set(false);
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

    fn set_z(&self, _z: i16) {}

    fn set_vcanvas(&self, _vcanvas_handle: usize) {}

    fn __event(&mut self, _context: BuildContext) {}
}
