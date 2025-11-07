/*
    Система компоновки UNWA (Useless name without abbreviation)

    Определяет компоновку элемента. Разделяется на 3 параметра
        LayoutType: Направление лайаута
        ContentAlignment: Как выравниваются элементы
        OverflowBehavior: Что происходит при выходе за Пределы контейнера
*/

// Направление лайаута, в какую сторону располагаются элеменьы
#[derive(Clone, Copy, Debug, Default)]
pub enum LayoutType {
    #[default]
    
    /*
        Элеиенты накладываются друг на друга как блинчики
        на тарелОчке, туда бы ещё сметанки
    */

    Stack,
    
    /*
        Элементы располагаются в ряд как солдаты на
        построении
    */
    
    Row,
    
    /*
        Элементы располагаются в вертикальную линюю
        и стоят как кирпичи,
    */

    Column,
}

// Как выравниваются элементы
#[derive(Clone, Copy, Debug, Default)]
pub enum ContentAlignment {
    #[default]
    Start,          // Элементы выравниваются к началу
    End,            // Элементы выравниваются к концу
    Center,         // Тоже самое, но к центру (Я устал это писать это всё равно понытно)
    Stretch,        // Растягивают на всё сводобное пространство
    SpaceBetween,   // Элементы распределяются так, что между ними одинаковое расстояние
}

// Что происходит когда элемент выходит за границы контейнера
#[derive(Clone, Copy, Debug, Default)]
pub enum OverflowBehavior {
    #[default]
    Clip,       // Обрезается, всё что не влезло не видно
    Visible,    // Может спокойно выходить за границы
    Wrap,       // Переносится на новую строку
}

#[derive(Clone, Debug)]
pub struct Unwa {
    pub layout: LayoutType,
    pub alignment: ContentAlignment,
    pub overflow: OverflowBehavior,

    /*
        Расстояние между элементами
        Для Stack: насколько они перекрываются
        Для Row: расстояние между детьми по горизонтали
        Для Column: расстояние между детьми по вертикали
    */

    pub gap: f32,

    /*
        Отступы от краёв контейнера, личное пространство которое
        не нарушают его дети (Как это мило :3 )
    */

    pub padding: f32,
}

impl Default for Unwa {
    fn default() -> Self {
        Self {
            layout: LayoutType::Column,
            alignment: ContentAlignment::Start,
            overflow: OverflowBehavior::Clip,
            gap: 0.0,
            padding: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Layout {
    Unwa(Unwa),
}

impl Default for Layout {
    fn default() -> Self {
        Layout::Unwa(Unwa::default())
    }
}