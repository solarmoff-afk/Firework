// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

mod common;

use crate::common::TestHarness;
use firework_ui::{ui, AdapterCommand};

#[ui]
fn test_spark_rect_screen() {
    let mut pos = spark!((10, 10)); // (i32, i32)

    rect! {
        position: pos, // Должна быть инициализация с (10, 10)
        color: (255, 255, 255),
    }

    pos = (20, 20); // Теперь это должно перезапустить реактивный цикл и переустановить
                    // позицию прямоугольника на (20, 20)
}

#[ui]
fn test_spark_conditional_rect_screen() {
    // Изначально его нет
    let mut condition = spark!(0); // i32

    // Условный рендеринг
    if condition == 1 {
        rect! {
            position: (10, 10),
            color: (255, 255, 255),
        }

        // Теперь условие снова не выполняется, это перезапустит цикл реактивности
        // и прямоугольник снова исчезнет
        condition = 2;
    }

    // Теперь он должен появится так как условие перезапустится
    condition = 1;
}

#[test]
fn test_spark_rect() { 
    let commands = TestHarness::run(test_spark_rect_screen);

    assert_eq!(commands, vec![
        AdapterCommand::RemoveAll,
        AdapterCommand::NewRect { layout: 1, },
        AdapterCommand::SetHitGroup(0, 65535),
        AdapterCommand::SetPosition(0, (10, 10)),
        AdapterCommand::SetColor(0, (255, 255, 255, 255)),
        AdapterCommand::SetPosition(0, (20, 20)),
    ]);
}

#[test]
fn test_spark_conditional_rect() { 
    let commands = TestHarness::run(test_spark_conditional_rect_screen);

    assert_eq!(commands, vec![
        AdapterCommand::RemoveAll,
        AdapterCommand::NewRect { layout: 1, },
        AdapterCommand::SetHitGroup(0, 65535),
        AdapterCommand::SetPosition(0, (10, 10)),
        AdapterCommand::SetColor(0, (255, 255, 255, 255)),

        // Промежуточного состояния (глитч condition = 1) не видно благодаря батчингу

        // Теперь его не видно (condition = 2)
        AdapterCommand::SetVisible(0, false),
    ]);
}
