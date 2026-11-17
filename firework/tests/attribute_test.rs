// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

mod common;

use crate::common::TestHarness;
use firework_adapter::TestCommand;
use firework_ui::ui;

type AdapterCommand = TestCommand;

/// #[raw] на let привязке отключает анализ выражения справа мутация x = 20 не делает
/// UpdateSpark, первый rect! остаётся на (10, 0), а второй rect! уже читает обновлённый
/// x = 20
#[ui]
fn test_raw_mutation_skips_reactivity_screen() {
    let mut x = spark!(10);

    rect! {
        position: (x, 0),
        color: (255, 255, 255),
    }

    // Иначе не работает в стабильном расте
    #[raw]
    let _ = {
        x = 20;
    };

    rect! {
        position: (x, 10),
        color: (0, 255, 0),
    }
}

#[test]
fn test_raw_mutation_skips_reactivity() {
    let commands = TestHarness::run(test_raw_mutation_skips_reactivity_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (10, 0)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (20, 10)),
            AdapterCommand::SetColor(0, (0, 255, 0, 255)),
            AdapterCommand::SetZ(0, 0),
            AdapterCommand::SetZ(0, 2),
        ]
    );
}

/// #[raw] отключает анализ вложенных циклов и условий внутри let блока
#[ui]
fn test_raw_control_flow_screen() {
    let mut sum = spark!(0);

    rect! {
        position: (sum, 0),
        color: (255, 0, 0),
    }

    #[raw]
    let _ = {
        for i in 1..4 {
            if i % 2 == 1 {
                sum += i;
            }
        }

        if sum == 4 {
            sum += 6;
        }
    };

    rect! {
        position: (sum, 20),
        color: (0, 0, 255),
    }
}

#[test]
fn test_raw_control_flow() {
    let commands = TestHarness::run(test_raw_control_flow_screen);

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
            AdapterCommand::SetPosition(0, (10, 20)),
            AdapterCommand::SetColor(0, (0, 0, 255, 255)),
            AdapterCommand::SetZ(0, 0),
            AdapterCommand::SetZ(0, 2),
        ]
    );
}

/// #[raw] на макрос стейтменте (Stmt::Macro) с firework_ui::raw!
#[ui]
fn test_raw_macro_stmt_screen() {
    let mut x = spark!(1);

    rect! {
        position: (x, 0),
        color: (10, 20, 30),
    }

    #[raw]
    firework_ui::raw! {
        x += 9;
    }

    rect! {
        position: (x, 0),
        color: (40, 50, 60),
    }
}

#[test]
fn test_raw_macro_stmt() {
    let commands = TestHarness::run(test_raw_macro_stmt_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (1, 0)),
            AdapterCommand::SetColor(0, (10, 20, 30, 255)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (10, 0)),
            AdapterCommand::SetColor(0, (40, 50, 60, 255)),
            AdapterCommand::SetZ(0, 0),
            AdapterCommand::SetZ(0, 2),
        ]
    );
}

/// #[raw] на обычной локальной переменной + последующее реактивное обновление спарка
#[ui]
fn test_raw_local_binding_screen() {
    let mut val = spark!(10);

    #[raw]
    let step = 5;

    rect! {
        position: (val, step),
        color: (255, 255, 255),
    }

    if val == 10 {
        val += step;
    }
}

#[test]
fn test_raw_local_binding() {
    let commands = TestHarness::run(test_raw_local_binding_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (10, 5)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),
            AdapterCommand::SetPosition(0, (15, 5)),
            AdapterCommand::SetZ(0, 0),
        ]
    );
}

/// #[capacity(N)] на виджете внутри цикла (в том числе больше дефолтных 64)
#[ui]
fn test_capacity_on_widget_screen() {
    let mut count = spark!(2);

    for i in 0..count {
        #[capacity(128)]
        rect! {
            position: (i * 10, 0),
            color: (255, 255, 255),

            #[key_type(i32)]
            key: i,
        }
    }

    if count == 2 {
        count = 3;
    }
}

#[test]
fn test_capacity_on_widget() {
    let commands = TestHarness::run(test_capacity_on_widget_screen);

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
            AdapterCommand::SetZ(0, 0),
            AdapterCommand::SetZ(0, 1),
            AdapterCommand::SetZ(0, 2),
        ]
    );
}

/// #[capacity(4)] при росте и уменьшении количества элементов в DynList
#[ui]
fn test_capacity_grow_and_shrink_screen() {
    let mut count = spark!(2);

    for i in 0..count {
        #[capacity(4)]
        rect! {
            position: (i * 10, 0),
            color: (255, 255, 255),

            #[key_type(i32)]
            key: i,
        }
    }

    if count == 2 {
        count = 4;
    } else if count == 4 {
        count = 1;
    }
}

#[test]
fn test_capacity_grow_and_shrink() {
    let commands = TestHarness::run(test_capacity_grow_and_shrink_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            // count = 2
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (0, 0)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (10, 0)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),
            // count = 4 (2 новых элемента)
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (20, 0)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (30, 0)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),
            // count = 1 (удаление 3 элементов)
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::Remove(0),
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::Remove(0),
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::Remove(0),
            AdapterCommand::SetZ(0, 0),
        ]
    );
}

/// Проверка что capacity не утекает в соседний цикл после выхода из стейтмента
#[ui]
fn test_capacity_restore_scope_screen() {
    let mut outer = spark!(2);

    for r in 0..outer {
        for c in 0..2 {
            #[capacity(4)]
            rect! {
                position: (r * 10, c * 10),
                color: (255, 255, 255),

                #[key_type((i32, i32))]
                key: (r, c),
            }
        }
    }

    for k in 0..outer {
        rect! {
            position: (k * 50, 100),
            color: (0, 255, 0),

            #[key_type(i32)]
            key: k,
        }
    }

    if outer == 2 {
        outer = 1;
    }
}

#[test]
fn test_capacity_restore_scope() {
    let commands = TestHarness::run(test_capacity_restore_scope_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            // Первый цикл: 4 прямоугольника (outer = 2, c = 0..2)
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
            // Второй цикл: 2 прямоугольника (outer = 2)
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (0, 100)),
            AdapterCommand::SetColor(0, (0, 255, 0, 255)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (50, 100)),
            AdapterCommand::SetColor(0, (0, 255, 0, 255)),
            // outer = 1: удаляются 2 элемента первого цикла и 1 элемент второго цикла
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::Remove(0),
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::Remove(0),
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::Remove(0),
            // Z-порядок: первый DynList (2 элемента -> 0, 1), второй DynList (1 элемент -> 3)
            AdapterCommand::SetZ(0, 0),
            AdapterCommand::SetZ(0, 1),
            AdapterCommand::SetZ(0, 3),
        ]
    );
}

/// #[key_type(usize)]
#[ui]
fn test_key_type_usize_screen() {
    let mut count = spark!(2usize);

    for i in 0..count {
        rect! {
            position: (i as i32 * 10, 0),
            color: (255, 255, 255),

            #[key_type(usize)]
            key: i,
        }
    }

    if count == 2 {
        count = 3;
    }
}

#[test]
fn test_key_type_usize() {
    let commands = TestHarness::run(test_key_type_usize_screen);

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
            AdapterCommand::SetZ(0, 0),
            AdapterCommand::SetZ(0, 1),
            AdapterCommand::SetZ(0, 2),
        ]
    );
}

/// #[key_type(String)] проверяет, что DynList переиспользует виджет при совпадении строки
#[ui]
fn test_key_type_string_screen() {
    let mut stage = spark!(0);

    for i in 0..2 {
        let key_str = if stage == 0 {
            format!("item_{}", i)
        } else {
            format!("item_{}", i + 1)
        };

        rect! {
            position: (i * 10, 0),
            color: (100, 150, 200),

            #[key_type(String)]
            key: key_str,
        }
    }

    if stage == 0 {
        stage = 1;
    }
}

#[test]
fn test_key_type_string() {
    let commands = TestHarness::run(test_key_type_string_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            // stage = 0: "item_0", "item_1"
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (0, 0)),
            AdapterCommand::SetColor(0, (100, 150, 200, 255)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (10, 0)),
            AdapterCommand::SetColor(0, (100, 150, 200, 255)),
            // stage = 1: "item_1" переиспользован, "item_2" создан, "item_0" удалён
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (10, 0)),
            AdapterCommand::SetColor(0, (100, 150, 200, 255)),
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::Remove(0),
            AdapterCommand::SetZ(0, 0),
            AdapterCommand::SetZ(0, 1),
        ]
    );
}

/// #[key_type((i32, bool))] с разнородным кортежем
#[ui]
fn test_key_type_heterogeneous_tuple_screen() {
    let mut flag = spark!(false);

    for i in 0..2 {
        rect! {
            position: (i * 20, 0),
            color: (255, 0, 0),

            #[key_type((i32, bool))]
            key: (i, flag),
        }
    }

    if !flag {
        flag = true;
    }
}

#[test]
fn test_key_type_heterogeneous_tuple() {
    let commands = TestHarness::run(test_key_type_heterogeneous_tuple_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            // (0, false), (1, false)
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (0, 0)),
            AdapterCommand::SetColor(0, (255, 0, 0, 255)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (20, 0)),
            AdapterCommand::SetColor(0, (255, 0, 0, 255)),
            // (0, true), (1, true) создаются, старые удаляются
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (0, 0)),
            AdapterCommand::SetColor(0, (255, 0, 0, 255)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (20, 0)),
            AdapterCommand::SetColor(0, (255, 0, 0, 255)),
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::Remove(0),
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::Remove(0),
            AdapterCommand::SetZ(0, 0),
            AdapterCommand::SetZ(0, 1),
        ]
    );
}

#[derive(PartialEq, Eq)]
struct ItemKey {
    group: u8,
    id: u32,
}

/// #[key_type(CustomStruct)] для пользовательского типа ключа с PartialEq + Eq
#[ui]
fn test_key_type_custom_struct_screen() {
    let mut keep_second = spark!(true);

    for i in 0..2u32 {
        if i == 0 || keep_second {
            rect! {
                position: (i as i32 * 30, 0),
                color: (0, 255, 0),

                #[key_type(ItemKey)]
                key: ItemKey { group: 1, id: i },
            }
        }
    }

    if keep_second {
        keep_second = false;
    }
}

#[test]
fn test_key_type_custom_struct() {
    let commands = TestHarness::run(test_key_type_custom_struct_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (0, 0)),
            AdapterCommand::SetColor(0, (0, 255, 0, 255)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (30, 0)),
            AdapterCommand::SetColor(0, (0, 255, 0, 255)),
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::Remove(0),
            AdapterCommand::SetVisible(0, true),
            AdapterCommand::SetZ(0, 0),
        ]
    );
}

#[ui]
fn test_attributes_combined_screen() {
    let mut count = spark!(3);

    #[raw]
    let color_g = 128u8;

    for i in 0..count {
        #[capacity(8)]
        rect! {
            position: (i * 15, 0),
            color: (0, color_g, 255),

            #[key_type((i32, i32))]
            key: (i, 1),
        }
    }

    #[raw]
    let raw_delta = 1;

    if count == 3 {
        count -= raw_delta;
    }
}

#[test]
fn test_attributes_combined() {
    let commands = TestHarness::run(test_attributes_combined_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (0, 0)),
            AdapterCommand::SetColor(0, (0, 128, 255, 255)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (15, 0)),
            AdapterCommand::SetColor(0, (0, 128, 255, 255)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (30, 0)),
            AdapterCommand::SetColor(0, (0, 128, 255, 255)),
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::Remove(0),
            AdapterCommand::SetZ(0, 0),
            AdapterCommand::SetZ(0, 1),
        ]
    );
}
