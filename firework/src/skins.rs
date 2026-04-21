// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use crate::adapter_command;
use firework_adapter::{AdapterCommand, AdapterResult};

#[derive(Debug, Clone, Copy)]
pub struct DefaultRectSkin {
    handle: usize,
    layout: u16,
}

impl DefaultRectSkin { 
    pub fn new(layout: u16) -> Option<Self> {
        match adapter_command(AdapterCommand::NewRect { layout }) {
            AdapterResult::Handle(handle) => Some(Self {
                handle: handle,
                layout,
            }),

            _ => None,
        }
    }

    /// Устанавливает позицию прямоугольника (левый верхний угол)
    pub fn position(self, x: i32, y: i32) -> Self {
        let _ = adapter_command(AdapterCommand::SetPosition(self.handle, (x, y)));
        self
    }

    /// Устанавливает размер прямоугольника
    pub fn size(self, width: i32, height: i32) -> Self {
        let _ = adapter_command(AdapterCommand::SetSize(self.handle, (width, height)));
        self
    }

    /// Устанавливает цвет прямоугольника
    pub fn color(self, r: u8, g: u8, b: u8) -> Self {
        let _ = adapter_command(AdapterCommand::SetColor(self.handle, (r, g, b, 255)));
        self
    }

    /// Устанавливает цвет прямоугольника с альфа-каналом
    pub fn color_alpha(self, r: u8, g: u8, b: u8, a: u8) -> Self {
        let _ = adapter_command(AdapterCommand::SetColor(self.handle, (r, g, b, a)));
        self
    }

    /// Устанавливает Z-индекс
    pub fn z(self, z: i32) -> Self {
        let _ = adapter_command(AdapterCommand::SetZ(self.handle, z));
        self
    }

    /// Устанавливает видимость прямоугольника
    pub fn visible(self, visible: bool) -> Self {
        let _ = adapter_command(AdapterCommand::SetVisible(self.handle, visible));
        self
    }

    /// Устанавливает хит-группу для обработки кликов
    pub fn hit_group(self, group: u16) -> Self {
        let _ = adapter_command(AdapterCommand::SetHitGroup(self.handle, group));
        self
    }

    /// Возвращает хэндл объекта
    pub fn handle(&self) -> usize {
        self.handle
    } 

    pub fn remove(self) {
        let _ = adapter_command(AdapterCommand::Remove(self.handle));
    }
}
