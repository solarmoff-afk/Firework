// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

// Conbine стал слишком большим поэтому новый тесты из этой категории будут добавлены именно
// сюда. Тут также будут комбинированные тесты как и в Conbine

mod common;

use crate::common::TestHarness;
use firework_ui::{AdapterCommand, ui};

#[ui]
fn test_combine_control_flow_screen() {
    let mut count = spark!(0);
    let mut skip_twos = spark!(false);

    for i in 0..count {
        if skip_twos && i == 2 {
            continue;
        }

        if i == 4 {
            break;
        }

        rect! {
            position: (i * 10, 0),
            color: (255, 255, 255),

            #[key_type(i32)]
            key: i,
        }
    }

    if count == 0 {
        count = 5;
    } else if !skip_twos {
        skip_twos = true;
    } else if count == 5 {
        count = 2;
    }
}

#[test]
fn test_combine_control_flow() {
    let commands = TestHarness::run(test_combine_control_flow_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (0, 0)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (10, 0)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (20, 0)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (30, 0)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::Remove(0),
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::Remove(0),
        ]
    );
}
