// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use crate::adapter_command;
use firework_adapter::{AdapterCommand, AdapterResult};

#[derive(Debug, Clone, Copy)]
pub struct DefaultRectSkin {
    handle: usize,
    _layout: u16,
}

impl DefaultRectSkin {
    pub fn new(layout: u16) -> Option<Self> {
        match adapter_command(AdapterCommand::NewRect { layout }) {
            AdapterResult::Handle(handle) => {
                adapter_command(AdapterCommand::SetHitGroup(handle, crate::TOUCH_HIT_GROUP));

                Some(Self {
                    handle: handle,
                    _layout: layout,
                })
            }

            _ => None,
        }
    }

    /// Устанавливает позицию прямоугольника (левый верхний угол)
    pub fn position(self, position: (i32, i32)) -> Self {
        let _ = adapter_command(AdapterCommand::SetPosition(self.handle, position));
        self
    }

    /// Устанавливает размер прямоугольника
    pub fn size(self, size: (i32, i32)) -> Self {
        let _ = adapter_command(AdapterCommand::SetSize(self.handle, size));
        self
    }

    /// Устанавливает цвет прямоугольника
    pub fn color(self, color: (u8, u8, u8)) -> Self {
        let _ = adapter_command(AdapterCommand::SetColor(
            self.handle,
            (color.0, color.1, color.2, 255),
        ));
        self
    }

    /// Устанавливает цвет прямоугольника с альфа-каналом
    pub fn color_alpha(self, color: (u8, u8, u8, u8)) -> Self {
        let _ = adapter_command(AdapterCommand::SetColor(self.handle, color));
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

    pub fn __id(&self) -> usize {
        self.handle
    }
}

impl crate::dyn_list::SkinVisibility for DefaultRectSkin {
    fn visible(&self, state: bool) {
        DefaultRectSkin::visible(*self, state);
    }

    fn unmount(self) {
        self.visible(false);

        crate::adapter_command(crate::AdapterCommand::Remove(self.__id()));
    }
}
