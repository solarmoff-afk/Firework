// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use firework_adapter::TestCommand;
use firework_ui::{
    AdapterClickPhase, BuildContext, CurrentEvent, Prop, component, dispatch_event, get_tick_fn,
    run_with_adapter, test_null_adapter, ui,
};
use std::cell::RefCell;

type AdapterCommand = TestCommand;

thread_local! {
    static TIMER_COMMANDS: RefCell<Vec<TestCommand>> = const { RefCell::new(Vec::new()) };
}

fn timer_adapter(command: firework_ui::AdapterCommand) -> firework_ui::AdapterResult {
    if !matches!(command, firework_ui::AdapterCommand::RunLoop { .. }) {
        TIMER_COMMANDS.with(|cmds| cmds.borrow_mut().push(TestCommand::from(command)));
    }
    test_null_adapter(command)
}

fn take_commands() -> Vec<TestCommand> {
    TIMER_COMMANDS.with(|cmds| std::mem::take(&mut *cmds.borrow_mut()))
}

fn click_screen() {
    dispatch_event(CurrentEvent::Touch {
        x: 0,
        y: 0,
        hit_object_id: Some(0),
        phase: AdapterClickPhase::Ended,
    });
}

#[ui]
fn test_timer_pause_resume_screen() {
    let mut running = spark!(true);
    let mut done = spark!(false);

    if running {
        timer! {
            delay: 100,
            target: done,
            output: true,
        }
    }

    rect! {
        position: (if done { 100 } else { 0 }, 0),
        color: (255, 255, 255),
        on_click: || {
            running = !running;
        },
    }
}

#[test]
fn test_timer_pause_and_resume_conditional() {
    let _ = take_commands();
    run_with_adapter(timer_adapter, test_timer_pause_resume_screen);

    assert_eq!(
        take_commands(),
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (0, 0)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),
            AdapterCommand::SetZ(0, 1),
        ]
    );

    get_tick_fn()(60);
    assert_eq!(take_commands(), vec![]);

    click_screen();
    assert_eq!(take_commands(), vec![]);

    get_tick_fn()(100);
    get_tick_fn()(100);
    assert_eq!(take_commands(), vec![]);

    click_screen();
    assert_eq!(take_commands(), vec![]);

    get_tick_fn()(50);
    assert_eq!(take_commands(), vec![]);

    get_tick_fn()(1);
    assert_eq!(
        take_commands(),
        vec![AdapterCommand::SetPosition(0, (100, 0))]
    );
}

#[ui]
fn test_timer_initially_hidden_screen() {
    let mut armed = spark!(false);
    let mut done = spark!(false);

    if armed {
        timer! {
            delay: 50,
            target: done,
            output: true,
        }
    }

    rect! {
        position: (if done { 50 } else { 0 }, 0),
        color: (0, 200, 0),
        on_click: || {
            armed = true;
        },
    }
}

#[test]
fn test_timer_initially_hidden_does_not_tick() {
    let _ = take_commands();
    run_with_adapter(timer_adapter, test_timer_initially_hidden_screen);
    let _ = take_commands();

    get_tick_fn()(200);
    get_tick_fn()(200);
    assert_eq!(take_commands(), vec![]);

    click_screen();
    assert_eq!(take_commands(), vec![]);

    get_tick_fn()(50);
    assert_eq!(take_commands(), vec![]);

    get_tick_fn()(1);
    assert_eq!(
        take_commands(),
        vec![AdapterCommand::SetPosition(0, (50, 0))]
    );
}

component! {
    pub struct DelayedBadge {
        pub active_x: Prop<i32>,
    }

    impl DelayedBadge {
        pub fn new() -> Self {
            Self { active_x: None }
        }

        pub fn flash(&mut self, _ctx: BuildContext) {
            let mut ready = spark!(false);

            timer! {
                delay: 80,
                target: ready,
                output: true,
            }

            rect! {
                position: (if ready { self.active_x.unwrap_or(0) } else { 0 }, 0),
                color: (255, 100, 0),
            }
        }
    }
}

#[ui]
fn test_timer_in_component_screen() {
    component! {
        target: DelayedBadge,
        active_x: 77,
    }
}

#[test]
fn test_timer_in_component() {
    let _ = take_commands();
    run_with_adapter(timer_adapter, test_timer_in_component_screen);

    assert_eq!(
        take_commands(),
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetColor(0, (255, 255, 255, 0)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (0, 0)),
            AdapterCommand::SetColor(0, (255, 100, 0, 255)),
            AdapterCommand::SharedVCanvas(0, 0),
            AdapterCommand::SetZ(0, 1),
            AdapterCommand::SetZ(0, 1),
            AdapterCommand::SetZ(0, 4),
        ]
    );

    get_tick_fn()(80);
    assert_eq!(take_commands(), vec![]);

    get_tick_fn()(1);
    assert_eq!(
        take_commands(),
        vec![AdapterCommand::SetPosition(0, (77, 0))]
    );
}

#[ui]
fn test_timer_component_conditional_pause_screen() {
    let mut show = spark!(true);

    if show {
        component! {
            target: DelayedBadge,
            active_x: 55,
        }
    }

    rect! {
        position: (0, 100),
        color: (0, 0, 255),
        on_click: || {
            show = !show;
        },
    }
}

#[test]
fn test_timer_in_conditional_component_pause() {
    let _ = take_commands();
    run_with_adapter(timer_adapter, test_timer_component_conditional_pause_screen);
    let _ = take_commands();

    get_tick_fn()(40);
    assert_eq!(take_commands(), vec![]);

    click_screen();
    assert_eq!(take_commands(), vec![AdapterCommand::SetVisible(0, false)]);

    get_tick_fn()(200);
    get_tick_fn()(200);
    assert_eq!(take_commands(), vec![]);

    click_screen();
    assert_eq!(take_commands(), vec![AdapterCommand::SetVisible(0, true)]);

    get_tick_fn()(40);
    assert_eq!(take_commands(), vec![]);

    get_tick_fn()(1);
    assert_eq!(
        take_commands(),
        vec![
            AdapterCommand::SetPosition(0, (55, 0)),
            AdapterCommand::SetVisible(0, true),
        ]
    );
}

#[ui]
fn test_timer_component_in_dynlist_screen() {
    let mut count = spark!(1);

    for i in 0..count {
        component! {
            target: DelayedBadge,
            active_x: 90,

            #[key_type(i32)]
            key: i,
        }
    }

    rect! {
        position: (0, 200),
        color: (128, 128, 128),
        on_click: || {
            count = 0;
        },
    }
}

#[test]
fn test_timer_component_in_dynlist_fires_and_unmounts() {
    let _ = take_commands();
    run_with_adapter(timer_adapter, test_timer_component_in_dynlist_screen);

    assert_eq!(
        take_commands(),
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetColor(0, (255, 255, 255, 0)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (0, 0)),
            AdapterCommand::SetColor(0, (255, 100, 0, 255)),
            AdapterCommand::SharedVCanvas(0, 0),
            AdapterCommand::SetZ(0, 1),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (0, 200)),
            AdapterCommand::SetColor(0, (128, 128, 128, 255)),
            AdapterCommand::SetZ(0, 1),
            AdapterCommand::SetZ(0, 4),
            AdapterCommand::SetZ(0, 2),
        ]
    );

    // Таймер внутри элемента DynList отрабатывает штатно
    get_tick_fn()(80);
    assert_eq!(take_commands(), vec![]);

    get_tick_fn()(1);
    assert_eq!(
        take_commands(),
        vec![
            AdapterCommand::SetPosition(0, (90, 0)),
            AdapterCommand::SetZ(0, 1),
            AdapterCommand::SetZ(0, 4),
            AdapterCommand::SetZ(0, 2),
        ]
    );

    // Удаляем элемент из DynList unmount гасит и удаляет компонент
    click_screen();
    assert_eq!(
        take_commands(),
        vec![
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::Remove(0),
            AdapterCommand::SetZ(0, 1),
        ]
    );

    // После unmount тики ничего не вызывают
    get_tick_fn()(100);
    get_tick_fn()(1);
    assert_eq!(take_commands(), vec![]);
}

#[ui]
fn test_timer_component_in_dynlist_unmount_stops_pending_timer_screen() {
    let mut count = spark!(1);

    for i in 0..count {
        component! {
            target: DelayedBadge,
            active_x: 90,

            #[key_type(i32)]
            key: i,
        }
    }

    rect! {
        position: (0, 200),
        color: (128, 128, 128),
        on_click: || {
            count = 0;
        },
    }
}

#[test]
fn test_timer_component_in_dynlist_unmount_stops_pending_timer() {
    let _ = take_commands();
    run_with_adapter(
        timer_adapter,
        test_timer_component_in_dynlist_unmount_stops_pending_timer_screen,
    );
    let _ = take_commands();

    get_tick_fn()(40);
    assert_eq!(take_commands(), vec![]);

    click_screen();
    assert_eq!(
        take_commands(),
        vec![
            AdapterCommand::SetVisible(0, false),
            AdapterCommand::Remove(0),
            AdapterCommand::SetZ(0, 1),
        ]
    );

    // Оставшееся время не должно вызвать срабатывание удалённого таймера
    get_tick_fn()(100);
    get_tick_fn()(1);
    assert_eq!(take_commands(), vec![]);
}
