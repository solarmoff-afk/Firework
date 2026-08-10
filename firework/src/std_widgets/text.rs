// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use crate::adapter_command;
use crate::layout::{Constraints, Size};
use crate::std_widgets::widget::Widget;
use firework_adapter::{AdapterCommand, AdapterResult};

#[derive(Debug, Clone, Copy)]
pub struct DefaultTextSkin {
    handle: usize,
    size: (i32, i32),
    _layout: u16,
}

impl DefaultTextSkin {
    pub fn new(layout: u16) -> Option<Self> {
        match adapter_command(AdapterCommand::NewText { layout }) {
            AdapterResult::Handle(handle) => {
                adapter_command(AdapterCommand::SetHitGroup(handle, crate::TOUCH_HIT_GROUP));

                Some(Self {
                    handle,
                    size: (0, 0),
                    _layout: layout,
                })
            }
            _ => None,
        }
    }

    pub fn position(self, position: (i32, i32)) -> Self {
        let _ = adapter_command(AdapterCommand::SetPosition(self.handle, position));
        self
    }

    pub fn text(self, text: impl AsRef<str>) -> Self {
        let _ = adapter_command(AdapterCommand::ClearText(self.handle));
        let _ = adapter_command(AdapterCommand::PushText {
            handle: self.handle,
            text: text.as_ref(),

            // TODO: Сделать выбор шрифта. Сейчас стандартный
            mode: 0,
        });
        self
    }

    pub fn font_size(self, size: u16) -> Self {
        let _ = adapter_command(AdapterCommand::SetFontSize(self.handle, size));
        self
    }

    pub fn color(self, color: (u8, u8, u8)) -> Self {
        let _ = adapter_command(AdapterCommand::SetColor(
            self.handle,
            (color.0, color.1, color.2, 255),
        ));
        self
    }

    pub fn visible(self, visible: bool) -> Self {
        let _ = adapter_command(AdapterCommand::SetVisible(self.handle, visible));
        self
    }

    pub fn __id(&self) -> usize {
        self.handle
    }
}

impl Widget for DefaultTextSkin {
    fn position(&self, position: (i32, i32)) {
        DefaultTextSkin::position(*self, position);
    }

    fn visible(&self, state: bool) {
        DefaultTextSkin::visible(*self, state);
    }

    fn unmount(self) {
        self.visible(false);
        crate::adapter_command(crate::AdapterCommand::Remove(self.__id()));
    }

    fn layout(&mut self, _constraints: Constraints) -> Size {
        match adapter_command(AdapterCommand::MeasureText(self.handle)) {
            AdapterResult::Size(w, h) => {
                self.size = (w as i32, h as i32);
            }
            _ => {
                self.size = (100, 20);
            }
        }

        Size {
            width: self.size.0,
            height: self.size.1,
        }
    }
}
