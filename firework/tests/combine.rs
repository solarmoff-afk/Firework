// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

mod common;

use crate::common::TestHarness;
use firework_ui::{ui, AdapterCommand};

#[ui]
fn test_combine_dynamic_filter_screen() {
    let mut filter_active = spark!(false);
    let mut items_count = spark!(0);

    for i in 0..items_count {
        if !filter_active || i % 2 == 0 {
            rect! {
                position: (10 * i, 10),
                color: (255, 255, 255),
                
                #[key_type(i32)]
                key: i,
            }
        }
    }

    if items_count == 0 {
        items_count = 3; 
    } else if !filter_active {
        filter_active = true; 
    } else if items_count == 3 {
        items_count = 1; 
    }
}

#[test]
fn test_combine_dynamic_filter() {
    let commands = TestHarness::run(test_combine_dynamic_filter_screen);

    assert_eq!(commands, vec![
        AdapterCommand::RemoveAll,
        
        AdapterCommand::NewRect { layout: 1 },
        AdapterCommand::SetHitGroup(0, 65535),
        AdapterCommand::SetPosition(0, (0, 10)),
        AdapterCommand::SetColor(0, (255, 255, 255, 255)),
        // Элемент i = 1
        AdapterCommand::NewRect { layout: 1 },
        AdapterCommand::SetHitGroup(0, 65535),
        AdapterCommand::SetPosition(0, (10, 10)),
        AdapterCommand::SetColor(0, (255, 255, 255, 255)),
        // Элемент i = 2
        AdapterCommand::NewRect { layout: 1 },
        AdapterCommand::SetHitGroup(0, 65535),
        AdapterCommand::SetPosition(0, (20, 10)),
        AdapterCommand::SetColor(0, (255, 255, 255, 255)),
        
        AdapterCommand::SetVisible(0, false),
        AdapterCommand::Remove(0),

        AdapterCommand::SetVisible(0, false),
        AdapterCommand::Remove(0),
    ]);
}

#[ui]
fn test_combine_nested_structures_screen() {
    let mut show_board = spark!(false);
    let mut rows = spark!(0);

    if show_board {
        for r in 0..rows {
            for c in 0..2 {
                rect! {
                    position: (r * 10, c * 10),
                    color: (255, 0, 0),
                    
                    #[key_type((i32, i32))]
                    key: (r, c),
                }
            }
        }
    }

    // Машина состояний
    if rows == 0 {
        rows = 2;
        show_board = true;
    } else if rows == 2 {
        rows = 1;
    } else if show_board {
        show_board = false;
    }
}

#[test]
fn test_combine_nested_structures() {
    let commands = TestHarness::run(test_combine_nested_structures_screen);

    assert_eq!(commands, vec![
        AdapterCommand::RemoveAll,
        
        // (r=0, c=0)
        AdapterCommand::NewRect { layout: 1 },
        AdapterCommand::SetHitGroup(0, 65535),
        AdapterCommand::SetPosition(0, (0, 0)),
        AdapterCommand::SetColor(0, (255, 0, 0, 255)),
        // (r=0, c=1)
        AdapterCommand::NewRect { layout: 1 },
        AdapterCommand::SetHitGroup(0, 65535),
        AdapterCommand::SetPosition(0, (0, 10)),
        AdapterCommand::SetColor(0, (255, 0, 0, 255)),
        // (r=1, c=0)
        AdapterCommand::NewRect { layout: 1 },
        AdapterCommand::SetHitGroup(0, 65535),
        AdapterCommand::SetPosition(0, (10, 0)),
        AdapterCommand::SetColor(0, (255, 0, 0, 255)),
        // (r=1, c=1)
        AdapterCommand::NewRect { layout: 1 },
        AdapterCommand::SetHitGroup(0, 65535),
        AdapterCommand::SetPosition(0, (10, 10)),
        AdapterCommand::SetColor(0, (255, 0, 0, 255)),

        AdapterCommand::SetVisible(0, false),
        AdapterCommand::Remove(0), // Для (1, 0)
        AdapterCommand::SetVisible(0, false),
        AdapterCommand::Remove(0), // Для (1, 1)

        AdapterCommand::SetVisible(0, false), // Для (0, 0)
        AdapterCommand::SetVisible(0, false), // Для (0, 1)
    ]);
}
