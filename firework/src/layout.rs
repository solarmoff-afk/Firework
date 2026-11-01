// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use crate::BuildContext;
use crate::std_widgets::widget::Widget;
use core::ops::{Deref, DerefMut};

/// Структура ограничений которые родитель отдаёт ребёнку. В Firework есть чёткое разделение
/// на лайаут и виджет поэтому лайаут берёт свои ограничения (которые он получил от своего
/// родителя или корня) и передаёт их детям чтобы они знали в какое пространство им нужно
/// вместиться
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Constraints {
    pub min_width: i32,
    pub max_width: i32,
    pub min_height: i32,
    pub max_height: i32,
}

impl Constraints {
    /// Создает жесткие ограничения (когда виджет обязан принять точный размер)
    pub fn tight(size: Size) -> Self {
        Self {
            min_width: size.width,
            max_width: size.width,
            min_height: size.height,
            max_height: size.height,
        }
    }

    /// Создает свободные ограничения (виджет может быть от 0 до заданного размера)
    pub fn loose(size: Size) -> Self {
        Self {
            min_width: 0,
            max_width: size.width,
            min_height: 0,
            max_height: size.height,
        }
    }

    /// Вгоняет запрошенный размер в рамки текущих ограничений
    pub fn constrain(&self, size: Size) -> Size {
        Size {
            width: size.width.clamp(self.min_width, self.max_width),
            height: size.height.clamp(self.min_height, self.max_height),
        }
    }

    /// Проверяет являются ли ограничения фиксированными
    pub fn is_tight(&self) -> bool {
        self.min_width == self.max_width && self.min_height == self.max_height
    }

    /// Безопасно уменьшает ограничения на заданный паддинг (top, right, bottom, left)
    pub fn deflate(&self, padding: (i32, i32, i32, i32)) -> Self {
        let horizontal = padding.3 + padding.1; // Лево и право
        let vertical = padding.0 + padding.2; // Верх и низ

        Self {
            min_width: (self.min_width - horizontal).max(0),
            max_width: (self.max_width - horizontal).max(0),
            min_height: (self.min_height - vertical).max(0),
            max_height: (self.max_height - vertical).max(0),
        }
    }
}

/// Итоговый размер виджета после вызова layout
#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

impl Size {
    pub fn zero() -> Self {
        Self {
            width: 0,
            height: 0,
        }
    }

    /// Проверка того что размер виджета не выходит за границы ограничений родительского
    /// лайаута
    pub fn is_valid(&self, constraints: Constraints) -> bool {
        // Ширина должна быть больше равна минимальных размеров и меньше равна максимальных
        let width_is_valid =
            self.width >= constraints.min_width && self.width <= constraints.max_width;

        // С высотой также
        let height_is_valid =
            self.height >= constraints.min_height && self.height <= constraints.max_height;

        // True возвращается если и ширина и высота воезают в ограчения
        width_is_valid && height_is_valid
    }
}

/// Параметры макета (лайаута) для компоновки, компилятор извлекает их из layout! {} виджета
/// и генерирует заполнение этой структуры
#[derive(Debug, Clone, Copy)]
pub struct LayoutParams {
    // Внутренний отступ контейнера
    pub padding: (
        /* top */ i32,
        /* right */ i32,
        /* bottom */ i32,
        /* left */ i32,
    ),
}

impl LayoutParams {
    /// Применяет LayoutParams к ограничениям (Constraints) и возвращает изменённые
    /// ограничения для детей. Min_width и min_height не меняются, работа идёт только
    /// с максимумом
    pub fn apply_to(&self, constraints: &Constraints) -> Constraints {
        let mut new_constraints = *constraints;

        // Обработка padding. Padding это внутренний отсуп внутри контейнера, то есть:
        //  {-------------}
        //  {  X       X  }
        //  {-------------}
        //   --         --
        // Padding    Padding
        //
        // Для этого нужно взять макисмальную ширину и высоту после чего отнять от них
        // нужный padding, то есть:
        //  - Для ширины: max_width - (left + right)
        //  - Для высоты: max_height - (top + bottom)

        new_constraints.max_width -= /* left */ self.padding.3 + /* Right */ self.padding.1;
        new_constraints.max_height -= /* Top */ self.padding.0 + /* Bottom */ self.padding.2;

        // Также если в результате этой операции максимальное ограничение станет меньше
        // минимального то нужно сделать его минимальным (max_width = min_width и для
        // height такую же операцию)
        new_constraints.max_width = new_constraints
            .max_width
            .max(new_constraints.min_width)
            .max(0);
        new_constraints.max_height = new_constraints
            .max_height
            .max(new_constraints.min_height)
            .max(0);

        new_constraints
    }
}

/// Ось компоновки
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

/// Выравнивание по главной оси
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainAxisAlignment {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Выравнивание по второй оси
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossAxisAlignment {
    Start,
    Center,
    End,
    Stretch,
}

/// Специальный контейнер для виджета который находится внутри лайаута. Он хранит внутри
/// кэшируемую информацию и флаг is_dirty. Не применяется для виджетов без отображения, то
/// есть таймеров, layout конфигуратора (layout! {}) и так далее
#[derive(Debug)]
pub struct LayoutState<T> {
    pub widget: T,
    pub cached_constraints: Constraints,
    pub cached_size: Size,
    pub cached_position: (i32, i32),
    pub is_dirty: bool,
}

impl<T: Clone> Clone for LayoutState<T> {
    fn clone(&self) -> Self {
        Self {
            widget: self.widget.clone(),
            cached_constraints: self.cached_constraints,
            cached_size: self.cached_size,
            cached_position: self.cached_position,
            is_dirty: self.is_dirty,
        }
    }
}

impl<T: Copy> Copy for LayoutState<T> {}

impl<T> LayoutState<T> {
    pub fn new(widget: T) -> Self {
        Self {
            widget,
            cached_constraints: Constraints {
                min_width: 0,
                max_width: 0,
                min_height: 0,
                max_height: 0,
            },
            cached_size: Size::zero(),
            cached_position: (0, 0),
            is_dirty: true,
        }
    }

    #[inline(always)]
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    #[inline(always)]
    pub fn needs_layout(&self, new_constraints: &Constraints) -> bool {
        self.is_dirty || &self.cached_constraints != new_constraints
    }
}

impl<T> Deref for LayoutState<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.widget
    }
}

impl<T> DerefMut for LayoutState<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget
    }
}

impl<T: Widget> Widget for LayoutState<T> {
    fn layout(&mut self, constraints: Constraints) -> Size {
        if !self.needs_layout(&constraints) {
            return self.cached_size;
        }

        let computed_size = self.widget.layout(constraints);

        self.cached_constraints = constraints;
        self.cached_size = computed_size;
        self.is_dirty = false;

        computed_size
    }

    fn position(&mut self, position: (i32, i32)) {
        if self.cached_position != position || self.is_dirty {
            self.cached_position = position;
            self.widget.position(position);
        }
    }

    #[inline(always)]
    fn visible(&mut self, state: bool) {
        self.widget.visible(state);
    }

    #[inline(always)]
    fn unmount(self) {
        self.widget.unmount();
    }

    #[inline(always)]
    fn get_z_size(&mut self) -> i16 {
        self.widget.get_z_size()
    }

    #[inline(always)]
    fn set_z(&mut self, z: i16) {
        self.widget.set_z(z);
    }

    #[inline(always)]
    fn set_vcanvas(&self, vcanvas_handle: usize) {
        self.widget.set_vcanvas(vcanvas_handle);
    }

    #[inline(always)]
    fn __event(&mut self, context: BuildContext) {
        self.widget.__event(context);
    }

    #[inline(always)]
    fn __tick(&mut self, delta: u32) -> bool {
        self.widget.__tick(delta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_size_is_valid() {
        // Квадратный родитель
        let constraints = Constraints {
            min_width: 0,
            min_height: 0,
            max_width: 100,
            max_height: 100,
        };

        // Занимает всего родителя
        let size = Size {
            width: 100,
            height: 100,
        };

        // Это валидно
        assert!(size.is_valid(constraints));

        // Шире родителя
        let size = Size {
            width: 200,
            height: 100,
        };

        // Это невалидно
        assert!(!size.is_valid(constraints));
    }
}
