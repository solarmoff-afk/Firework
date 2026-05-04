// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

mod common;

use crate::common::TestHarness;
use firework_ui::{AdapterCommand, ui};

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

    assert_eq!(
        commands,
        vec![
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
            AdapterCommand::SetVisible(0, true),
        ]
    );
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

    assert_eq!(
        commands,
        vec![
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
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::Remove(0),
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::Remove(0),
        ]
    );
}

#[ui]
fn test_combine_toggle_visibility_screen() {
    let mut show_main = spark!(false);
    let mut show_details = spark!(false);

    if show_main {
        rect! {
            position: (0, 0),
            color: (100, 100, 100),

            #[key_type(i32)]
            key: 1,
        }

        if show_details {
            rect! {
                position: (0, 50),
                color: (200, 200, 200),

                #[key_type(i32)]
                key: 2,
            }
        }
    }

    // Стейт-машина
    if !show_main {
        show_main = true;
    } else if !show_details {
        show_details = true;
    } else if show_main {
        show_main = false;
    }
}

#[test]
fn test_combine_toggle_visibility() {
    let commands = TestHarness::run(test_combine_toggle_visibility_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (0, 0)),
            AdapterCommand::SetColor(0, (100, 100, 100, 255)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (0, 50)),
            AdapterCommand::SetColor(0, (200, 200, 200, 255)),
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::SetVisible(0, false),
        ]
    );
}

#[ui]
fn test_combine_sibling_loops_screen() {
    let mut count_a = spark!(0);
    let mut count_b = spark!(0);

    for a in 0..count_a {
        rect! {
            position: (a * 10, 0),
            color: (255, 0, 0),

            #[key_type(i32)]
            key: a,
        }
    }

    for b in 0..count_b {
        rect! {
            position: (b * 10, 50),
            color: (0, 255, 0),

            #[key_type(i32)]
            key: b,
        }
    }

    if count_a == 0 {
        count_a = 2;
    } else if count_b == 0 {
        count_b = 1;
    } else if count_a == 2 {
        count_a = 1;
    }
}

#[test]
fn test_combine_sibling_loops() {
    let commands = TestHarness::run(test_combine_sibling_loops_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (0, 0)),
            AdapterCommand::SetColor(0, (255, 0, 0, 255)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (10, 0)),
            AdapterCommand::SetColor(0, (255, 0, 0, 255)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (0, 50)),
            AdapterCommand::SetColor(0, (0, 255, 0, 255)),
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::Remove(0),
        ]
    );
}

#[ui]
fn test_combine_advanced_reconciliation_screen() {
    let mut items_count = spark!(0);
    let mut filter_even = spark!(false);
    let mut global_offset = spark!(0);

    if items_count > 0 {
        for i in 0..items_count {
            if !filter_even || i % 2 == 0 {
                rect! {
                    position: (global_offset + i * 10, 0),
                    color: (255, 255, 255),

                    #[key_type(i32)]
                    key: i,
                }
            }
        }
    }

    if items_count == 0 {
        items_count = 3;
    } else if global_offset == 0 {
        global_offset = 100;
    } else if !filter_even {
        filter_even = true;
    } else if filter_even && items_count == 3 {
        items_count = 5;
    }
}

#[test]
fn test_combine_advanced_reconciliation() {
    let commands = TestHarness::run(test_combine_advanced_reconciliation_screen);

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
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::Remove(0),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (140, 0)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),
            AdapterCommand::SetVisible(0, true),
            AdapterCommand::SetVisible(0, true),
            AdapterCommand::SetVisible(0, true),
        ]
    );
}

#[ui]
fn test_combine_mixed_static_dynamic_screen() {
    let mut count = spark!(2);

    rect! {
        position: (0, 0),
        color: (255, 0, 0),
    }

    for i in 0..count {
        rect! {
            position: (10, i * 10),
            color: (0, 255, 0),

            #[key_type(i32)]
            key: i,
        }
    }

    rect! {
        position: (20, 0),
        color: (0, 0, 255),
    }

    // Машина состояний
    if count == 2 {
        count = 1; // Удалится только один зеленый прямоугольник
    }
}

#[test]
fn test_combine_mixed_static_dynamic() {
    let commands = TestHarness::run(test_combine_mixed_static_dynamic_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (0, 0)),
            AdapterCommand::SetColor(0, (255, 0, 0, 255)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (10, 0)),
            AdapterCommand::SetColor(0, (0, 255, 0, 255)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (10, 10)),
            AdapterCommand::SetColor(0, (0, 255, 0, 255)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (20, 0)),
            AdapterCommand::SetColor(0, (0, 0, 255, 255)),
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::Remove(0),
        ]
    );
}

#[ui]
fn test_combine_matrix_filter_screen() {
    let mut size = spark!(2);
    let mut show_diagonal = spark!(false);

    for r in 0..size {
        for c in 0..size {
            // Рисуем всё либо только диагональ
            if !show_diagonal || r == c {
                rect! {
                    position: (r * 10, c * 10),
                    color: (255, 255, 255),

                    #[key_type((i32, i32))]
                    key: (r, c),
                }
            }
        }
    }

    if !show_diagonal {
        show_diagonal = true;
    } else if size == 2 {
        size = 3;
    }
}

#[test]
fn test_combine_matrix_filter() {
    let commands = TestHarness::run(test_combine_matrix_filter_screen);

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
            AdapterCommand::SetPosition(0, (0, 10)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (10, 0)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (10, 10)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::Remove(0),
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::Remove(0),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (20, 20)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),
            AdapterCommand::SetVisible(0, true),
            AdapterCommand::SetVisible(0, true),
            AdapterCommand::SetVisible(0, true),
        ]
    );
}

#[ui]
fn test_combine_zero_to_hero_screen() {
    let mut count = spark!(0);

    for i in 0..count {
        rect! {
            position: (i * 10, 0),
            color: (255, 255, 0),

            #[key_type(i32)]
            key: i,
        }
    }

    if count == 0 {
        count = 2;
    } else if count == 2 {
        count = -1;
    }
}

#[test]
fn test_combine_zero_to_hero() {
    let commands = TestHarness::run(test_combine_zero_to_hero_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (0, 0)),
            AdapterCommand::SetColor(0, (255, 255, 0, 255)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (10, 0)),
            AdapterCommand::SetColor(0, (255, 255, 0, 255)),
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::Remove(0),
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::Remove(0),
        ]
    );
}
