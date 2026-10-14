// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::widget::Widget;

use crate::BuildContext;
use crate::layout::{Constraints, Size};

use core::cell::Cell;

#[derive(Debug)]
pub struct Timer<T> {
    pub time: Cell<u32>,
    pub delay: Cell<u32>,
    visible: Cell<bool>,
    pub is_active: Cell<bool>,
    pub output_value: Option<T>,
    pub id: Cell<(/* id */ usize, /* depth */ u16)>,
}

impl<T> Timer<T> {
    pub fn new(_layout: u16) -> Option<Self> {
        Some(Self {
            time: Cell::new(0),
            delay: Cell::new(0),
            visible: Cell::new(false),
            is_active: Cell::new(false),
            output_value: None,
            id: Cell::new((0, 0)),
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

    pub fn delay(self, delay: u32) -> Self {
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

    /// Конфигурация таймера
    pub fn __fwc_timer_cfg(self, id: usize, depth: u16) -> Self {
        self.id.set((id, depth));
        self
    }
}

impl<T> Widget for Timer<T> {
    fn position(&mut self, _position: (i32, i32)) {}

    fn visible(&mut self, state: bool) {
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

    fn get_z_size(&mut self) -> i16 {
        0
    }

    fn set_z(&mut self, _z: i16) {}

    fn set_vcanvas(&self, _vcanvas_handle: usize) {}

    fn __event(&mut self, _context: BuildContext) {}

    fn __tick(&mut self, delta: u32) -> bool {
        if !self.visible.get() || !self.is_active.get() {
            return false;
        }

        if self.time.get() < self.delay.get() {
            self.time.set(self.time.get() + delta);
            return false;
        }

        self.is_active.set(false);
        self.visible.set(false);

        true
    }
}
