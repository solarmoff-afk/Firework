// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

/// Структура ограничений которые родитель отдаёт ребёнку. В Firework есть чёткое разделение
/// на лайаут и виджет поэтому лайаут берёт свои ограничения (которые он получил от своего
/// родителя или корня) и передаёт их детям чтобы они знали в какое пространство им нужно
/// вместиться
#[derive(Debug, Clone, Copy)]
pub struct Constraints {
    pub min_width: i32,
    pub max_width: i32,
    pub min_height: i32,
    pub max_height: i32,
}

/// Итоговый размер виджета после вызова layout
#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

impl Size {
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
