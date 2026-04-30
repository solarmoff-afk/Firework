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

#[ui]
fn test_spark_derived_rect_screen() {
    let mut x = spark!(10);
    let mut pos = spark!((10, 10)); // (i32, i32)

    // Вычислительный спарк. Эффект без зависимостей защищает от бесконечного
    // цикла
    pos.0 = x;

    rect! {
        position: pos, // Должна быть инициализация с (10, 10)
        color: (255, 255, 255),
    }

    x = 20; // Должно вызвать реакцию с Pos и будет (20, 10)
}

#[ui]
fn test_spark_dynamic_rect_screen() {
    let mut count = spark!(3);  

    for i in 0..count {
        rect! {
            position: (10, 10),
            color: (255, 255, 255),
            
            #[key_type(i32)]
            key: i,
        }
    }

    count += 1;
}

#[ui]
fn test_spark_dynamic_decrement_rect_screen() {
    let mut count = spark!(3);  

    for i in 0..count {
        rect! {
            position: (10, 10),
            color: (255, 255, 255),
            
            #[key_type(i32)]
            key: i,
        }
    }

    // Один виджет удаляется
    count -= 1;
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

#[test]
fn test_spark_derived_rect() { 
    let commands = TestHarness::run(test_spark_derived_rect_screen);

    assert_eq!(commands, vec![
        AdapterCommand::RemoveAll,
        AdapterCommand::NewRect { layout: 1, },
        AdapterCommand::SetHitGroup(0, 65535),
        AdapterCommand::SetPosition(0, (10, 10)),
        AdapterCommand::SetColor(0, (255, 255, 255, 255)),
        AdapterCommand::SetPosition(0, (20, 10)),

        // ???
        AdapterCommand::SetPosition(0, (20, 10)),
    ]);
}

#[test]
fn test_spark_dynamic_rect() {
    let commands = TestHarness::run(test_spark_dynamic_rect_screen);

    assert_eq!(commands, vec![
        AdapterCommand::RemoveAll,
        
        // for i in 0..count {
        //  // Iter 1
        AdapterCommand::NewRect { layout: 1, },
        AdapterCommand::SetHitGroup(0, 65535),
        AdapterCommand::SetPosition(0, (10, 10)),
        AdapterCommand::SetColor(0, (255, 255, 255, 255)),
        //  // Iter 2 
        AdapterCommand::NewRect { layout: 1, },
        AdapterCommand::SetHitGroup(0, 65535),
        AdapterCommand::SetPosition(0, (10, 10)),
        AdapterCommand::SetColor(0, (255, 255, 255, 255)),

        //  // Iter 3
        AdapterCommand::NewRect { layout: 1, },
        AdapterCommand::SetHitGroup(0, 65535),
        AdapterCommand::SetPosition(0, (10, 10)),
        AdapterCommand::SetColor(0, (255, 255, 255, 255)),
        // }
        
        // Здесь происходит count += 1, теперь count = 4, цикл перезапускается, но создаётся только
        // один новый прямоугольник потому-что ключи оптимизируют создание. DynList видит что создан
        // только 1 прямоугольник и вызывает конструктор только для него
        
        //  // Iter 4
        AdapterCommand::NewRect { layout: 1, },
        AdapterCommand::SetHitGroup(0, 65535),
        AdapterCommand::SetPosition(0, (10, 10)),
        AdapterCommand::SetColor(0, (255, 255, 255, 255)),
    ]);
}

#[test]
fn test_spark_dynamic_decrement_rect() {
    let commands = TestHarness::run(test_spark_dynamic_decrement_rect_screen);
    
    assert_eq!(commands, vec![
        AdapterCommand::RemoveAll,
        
        // for i in 0..count {
        //  // Iter 1
        AdapterCommand::NewRect { layout: 1, },
        AdapterCommand::SetHitGroup(0, 65535),
        AdapterCommand::SetPosition(0, (10, 10)),
        AdapterCommand::SetColor(0, (255, 255, 255, 255)),

        //  // Iter 2 
        AdapterCommand::NewRect { layout: 1, },
        AdapterCommand::SetHitGroup(0, 65535),
        AdapterCommand::SetPosition(0, (10, 10)),
        AdapterCommand::SetColor(0, (255, 255, 255, 255)),

        //  // Iter 3
        AdapterCommand::NewRect { layout: 1, },
        AdapterCommand::SetHitGroup(0, 65535),
        AdapterCommand::SetPosition(0, (10, 10)),
        AdapterCommand::SetColor(0, (255, 255, 255, 255)),

        // }

        // count -= 1
        // Это размонтирует только последний виджет из DynList, он исчезнет и будет
        // размонтирован через команду Remove(id)
        AdapterCommand::SetVisible(0, false),
        AdapterCommand::Remove(0),
    ]);
}
