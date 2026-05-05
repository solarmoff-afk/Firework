// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

mod common;

use crate::common::TestHarness;
use firework_ui::{AdapterCommand, ui};

#[ui]
fn test_ui_rect_screen() {
    rect! {
        position: (10, 10),
        color: (255, 255, 255),
    }
}

#[ui]
fn test_ui_dynamic_rect_test() {
    for i in 0..3 {
        rect! {
            position: (10 * i, 10),
            color: (255, 255, 255),
            key: i as u64,
        }
    }
}

#[test]
fn test_ui_rect() {
    let commands = TestHarness::run(test_ui_rect_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (10, 10)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),
        ]
    );
}

#[test]
fn test_ui_dynamic_rect() {
    let commands = TestHarness::run(test_ui_dynamic_rect_test);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (0, 10)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),
            // В NullAdapter системы айди нет
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (10, 10)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (20, 10)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),
        ]
    );
}

#[ui]
fn test_ui_complex_properties_screen() {
    let mut mode = spark!(0);

    rect! {
        position: {
            let offset = 10;
            if mode == 0 {
                (offset, offset)
            } else {
                (offset * 5, offset * 5)
            }
        },
        color: match mode {
            0 => (255, 0, 0),
            _ => (0, 255, 0),
        },
    }

    if mode == 0 {
        mode = 1;
    }
}

#[test]
fn test_ui_complex_properties() {
    let commands = TestHarness::run(test_ui_complex_properties_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (10, 10)),
            AdapterCommand::SetColor(0, (255, 0, 0, 255)),
            AdapterCommand::SetPosition(0, (50, 50)),
            AdapterCommand::SetColor(0, (0, 255, 0, 255)),
        ]
    );
}
