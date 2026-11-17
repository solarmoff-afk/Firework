// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

mod common;

use crate::common::TestHarness;
use firework_adapter::TestCommand;
use firework_ui::{
    AdapterClickPhase, CurrentEvent, dispatch_event, get_tick_fn, run_with_adapter,
    test_null_adapter, ui,
};
use std::cell::RefCell;

type AdapterCommand = TestCommand;

thread_local! {
    static STREAM_COMMANDS: RefCell<Vec<TestCommand>> = const { RefCell::new(Vec::new()) };
}

fn stream_adapter(command: firework_ui::AdapterCommand) -> firework_ui::AdapterResult {
    if !matches!(command, firework_ui::AdapterCommand::RunLoop { .. }) {
        STREAM_COMMANDS.with(|cmds| cmds.borrow_mut().push(TestCommand::from(command)));
    }
    test_null_adapter(command)
}

fn take_stream_commands() -> Vec<TestCommand> {
    STREAM_COMMANDS.with(|cmds| std::mem::take(&mut *cmds.borrow_mut()))
}

// Эффект

#[ui]
fn test_effect_reactive_screen() {
    let mut trigger = spark!(1);
    let mut derived = spark!(0);

    // Реактивный эффект, он срабатывает при Build и при изменении trigger
    effect!(trigger, {
        derived = trigger * 10;
    });

    rect! {
        position: (derived, 0),
        color: (255, 255, 255),
    }

    if trigger == 1 {
        trigger = 3;
    }
}

#[test]
fn test_effect_reactive() {
    let commands = TestHarness::run(test_effect_reactive_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (10, 0)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),
            AdapterCommand::SetPosition(0, (30, 0)),
            AdapterCommand::SetPosition(0, (30, 0)),
            AdapterCommand::SetZ(0, 0),
        ]
    );
}

#[ui]
fn test_effect_multiple_deps_batching_screen() {
    let mut a = spark!(1);
    let mut b = spark!(2);
    let mut sum = spark!(0);

    // При изменении и a и b за один проход эффект срабатывает один раз без глича
    effect!(a, b, {
        sum = a + b;
    });

    rect! {
        position: (sum, 0),
        color: (255, 255, 255),
    }

    if a == 1 {
        a = 10;
        b = 20;
    }
}

#[test]
fn test_effect_multiple_deps_batching() {
    let commands = TestHarness::run(test_effect_multiple_deps_batching_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (3, 0)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),
            AdapterCommand::SetPosition(0, (30, 0)),
            AdapterCommand::SetPosition(0, (30, 0)),
            AdapterCommand::SetZ(0, 0),
        ]
    );
}

#[ui]
fn test_effect_inside_condition_screen() {
    let mut enabled = spark!(false);
    let mut value = spark!(1);
    let mut out = spark!(0);

    if enabled {
        effect!(value, {
            out = value * 100;
        });
    }

    rect! {
        position: (out, 0),
        color: (0, 255, 0),
    }

    if !enabled {
        enabled = true;
    }
    if enabled && value == 1 {
        value = 2;
    }
}

#[test]
fn test_effect_inside_condition() {
    let commands = TestHarness::run(test_effect_inside_condition_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (0, 0)),
            AdapterCommand::SetColor(0, (0, 255, 0, 255)),
            AdapterCommand::SetPosition(0, (200, 0)),
            AdapterCommand::SetZ(0, 0),
        ]
    );
}

// События

#[ui]
fn test_events_screen() {
    let mut clicks = spark!(0);
    let mut touches = spark!(0);

    rect! {
        position: (clicks * 10, touches * 10),
        size: (50, 50),
        color: (255, 0, 0),
        on_click: || {
            clicks += 1;
        },
        on_touch: |ev: firework_ui::TouchEvent| {
            if ev.is_began() || ev.is_ended() {
                touches += 1;
            }
        },
    }
}

#[test]
fn test_events_on_click_and_on_touch() {
    let _ = take_stream_commands();
    run_with_adapter(stream_adapter, test_events_screen);

    assert_eq!(
        take_stream_commands(),
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (0, 0)),
            AdapterCommand::SetSize(0, (50, 50)),
            AdapterCommand::SetColor(0, (255, 0, 0, 255)),
            AdapterCommand::SetZ(0, 0),
        ]
    );

    // Began срабатывает только on_touch (touches = 1), on_click игнорирует Began
    dispatch_event(CurrentEvent::Touch {
        x: 10,
        y: 10,
        hit_object_id: Some(0),
        phase: AdapterClickPhase::Began,
    });
    assert_eq!(
        take_stream_commands(),
        vec![AdapterCommand::SetPosition(0, (0, 10))]
    );

    // Ended срабатывают и on_click (clicks = 1) и on_touch (touches = 2)
    dispatch_event(CurrentEvent::Touch {
        x: 10,
        y: 10,
        hit_object_id: Some(0),
        phase: AdapterClickPhase::Ended,
    });
    assert_eq!(
        take_stream_commands(),
        vec![AdapterCommand::SetPosition(0, (10, 20))]
    );

    // Промах (hit_object_id: None) обработчики не срабатывают
    dispatch_event(CurrentEvent::Touch {
        x: 300,
        y: 300,
        hit_object_id: None,
        phase: AdapterClickPhase::Ended,
    });
    assert_eq!(take_stream_commands(), vec![]);
}

// Таймер

#[ui]
fn test_timer_screen() {
    let mut done = spark!(false);
    let mut extra = spark!(0);

    timer! {
        delay: 100,
        target: done,
        output: true,
        on_timeout: || {
            extra = 42;
        },
    }

    if done {
        rect! {
            position: (extra, 0),
            color: (0, 255, 0),
        }
    }
}

#[test]
fn test_timer_lifecycle() {
    let _ = take_stream_commands();
    run_with_adapter(stream_adapter, test_timer_screen);

    assert_eq!(take_stream_commands(), vec![AdapterCommand::RemoveAll]);

    // Тик 1 тут time = 60 (< 100)
    get_tick_fn()(60);
    assert_eq!(take_stream_commands(), vec![]);

    // Тик 2 тут time = 120 (>= 100), сработает на следующем тике
    get_tick_fn()(60);
    assert_eq!(take_stream_commands(), vec![]);

    // Тик 3 тут таймер завершается и диспатчит CurrentEvent::Timeout
    get_tick_fn()(1);
    assert_eq!(
        take_stream_commands(),
        vec![
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (42, 0)),
            AdapterCommand::SetColor(0, (0, 255, 0, 255)),
            AdapterCommand::SetPosition(0, (42, 0)),
            AdapterCommand::SetVisible(0, true),
        ]
    );
}

// Текст с реактивным форматом

#[ui]
fn test_text_braced_format_screen() {
    let mut score = spark!(10);

    text! {
        text: format!("Score: {score}"),
        position: (0, 0),
        color: (255, 255, 255),
    }

    if score == 10 {
        score = 42;
    }
}

#[test]
fn test_text_braced_format() {
    let commands = TestHarness::run(test_text_braced_format_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewText { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::ClearText(0),
            AdapterCommand::PushText {
                handle: 0,
                text: "Score: 10".to_string(),
                mode: 0,
            },
            AdapterCommand::MeasureText(0),
            AdapterCommand::SetPosition(0, (0, 0)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),
            AdapterCommand::ClearText(0),
            AdapterCommand::PushText {
                handle: 0,
                text: "Score: 42".to_string(),
                mode: 0,
            },
            AdapterCommand::MeasureText(0),
            AdapterCommand::SetZ(0, 0),
        ]
    );
}

#[ui]
fn test_while_loop_dynamic_screen() {
    let mut limit = spark!(2);

    let mut idx = 0;
    while idx < limit {
        rect! {
            position: (idx * 10, 0),
            color: (200, 100, 50),

            #[key_type(i32)]
            key: idx,
        }
        idx += 1;
    }

    if limit == 2 {
        limit = 1;
    }
}

#[test]
fn test_while_loop_dynamic() {
    let commands = TestHarness::run(test_while_loop_dynamic_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (0, 0)),
            AdapterCommand::SetColor(0, (200, 100, 50, 255)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (10, 0)),
            AdapterCommand::SetColor(0, (200, 100, 50, 255)),
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::Remove(0),
            AdapterCommand::SetZ(0, 0),
        ]
    );
}

// Ранний return

#[ui]
fn test_early_return_screen() {
    let mut step = spark!(0);

    rect! {
        position: (step * 10, 0),
        color: (255, 255, 255),
    }

    if step == 0 {
        step = 5;
        return;
    }

    rect! {
        position: (99, 99),
        color: (0, 0, 0),
    }
}

#[test]
fn test_early_return_preserves_spark() {
    let _ = take_stream_commands();
    run_with_adapter(stream_adapter, test_early_return_screen);

    // Первый вызов вернулся досрочно после обновления step = 5
    assert_eq!(
        take_stream_commands(),
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (0, 0)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),
        ]
    );

    // Повторный проход step был корректно возвращён в статику (= 5) и доходит до второго
    // rect!
    dispatch_event(CurrentEvent::None);
    assert_eq!(
        take_stream_commands(),
        vec![
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (99, 99)),
            AdapterCommand::SetColor(0, (0, 0, 0, 255)),
        ]
    );
}

// Навигация между экранами

#[ui]
fn nav_screen_a() {
    let mut counter = spark!(1);

    rect! {
        position: (counter, 0),
        color: (255, 0, 0),
        on_click: || {
            counter += 1;
        },
    }

    if counter == 1 {
        counter = 2;
    }
}

#[ui]
fn nav_screen_b() {
    rect! {
        position: (99, 99),
        color: (0, 0, 255),
    }
}

#[test]
fn test_screen_navigation_build_and_navigate() {
    let _ = take_stream_commands();

    // Build экрана A: counter 1 -> 2
    run_with_adapter(stream_adapter, nav_screen_a);
    assert_eq!(
        take_stream_commands(),
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (1, 0)),
            AdapterCommand::SetColor(0, (255, 0, 0, 255)),
            AdapterCommand::SetPosition(0, (2, 0)),
            AdapterCommand::SetZ(0, 0),
        ]
    );

    // Переход на экран B (Build)
    nav_screen_b();
    assert_eq!(
        take_stream_commands(),
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (99, 99)),
            AdapterCommand::SetColor(0, (0, 0, 255, 255)),
            AdapterCommand::SetZ(0, 0),
        ]
    );

    // Возврат на экран A
    nav_screen_a();
    assert_eq!(
        take_stream_commands(),
        vec![AdapterCommand::RemoveAll, AdapterCommand::SetZ(0, 0)]
    );

    // Клик на экране A
    dispatch_event(CurrentEvent::Touch {
        x: 0,
        y: 0,
        hit_object_id: Some(0),
        phase: AdapterClickPhase::Ended,
    });

    assert_eq!(
        take_stream_commands(),
        vec![AdapterCommand::SetPosition(0, (3, 0))]
    );
}

#[ui]
fn test_match_with_guard_screen() {
    let mut val = spark!(2);

    match val {
        x if x > 5 => {
            rect! {
                position: (x, 0),
                color: (0, 255, 0),
            }
        }
        _ => {
            rect! {
                position: (0, 0),
                color: (255, 0, 0),
            }
        }
    }

    if val == 2 {
        val = 10;
    }
}

#[test]
fn test_match_with_guard() {
    let commands = TestHarness::run(test_match_with_guard_screen);

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
            AdapterCommand::SetVisible(0, true),
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::SetZ(0, 0),
            AdapterCommand::SetZ(0, 2),
        ]
    );
}

// Свойства rect

#[ui]
fn test_rect_dimensions_and_alpha_screen() {
    let mut w = spark!(40);

    rect! {
        position: (5, 5),
        width: w,
        height: 80,
        color_alpha: (10, 20, 30, 128),
    }

    if w == 40 {
        w = 120;
    }
}

#[test]
fn test_rect_dimensions_and_alpha() {
    let commands = TestHarness::run(test_rect_dimensions_and_alpha_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (5, 5)),
            AdapterCommand::SetSize(0, (40, 0)),
            AdapterCommand::SetSize(0, (40, 80)),
            AdapterCommand::SetColor(0, (10, 20, 30, 128)),
            AdapterCommand::SetSize(0, (120, 80)),
            AdapterCommand::SetZ(0, 0),
        ]
    );
}
