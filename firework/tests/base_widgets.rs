// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use std::cell::RefCell;
use firework_ui::{ui, run_with_adapter, null_adapter, AdapterCommand, AdapterResult};

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

thread_local! {
    static COMMANDS: RefCell<Vec<AdapterCommand>> = RefCell::new(Vec::new());
}

fn adapter_handler(command: AdapterCommand) -> AdapterResult {
    if !matches!(command, AdapterCommand::RunLoop { .. } ) {
        println!("Command: {:#?}", command);
        COMMANDS.with(|cmds| cmds.borrow_mut().push(command));
    }

    null_adapter(command)
}

#[test]
fn test_ui_rect() { 
    COMMANDS.with(|cmds| cmds.borrow_mut().clear());
    
    run_with_adapter(adapter_handler, test_ui_rect_screen);
    
    COMMANDS.with(|cmds| {
        assert_eq!(*cmds.borrow(), vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1, },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (10, 10)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),
        ]);
    });
}

#[test]
fn test_ui_dynamic_rect() { 
    COMMANDS.with(|cmds| cmds.borrow_mut().clear());
    
    run_with_adapter(adapter_handler, test_ui_dynamic_rect_test);
    
    COMMANDS.with(|cmds| {
        assert_eq!(*cmds.borrow(), vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1, },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (0, 10)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),

            // В NullAdapter системы айди нет
            AdapterCommand::NewRect { layout: 1, },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (10, 10)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),

            AdapterCommand::NewRect { layout: 1, },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (20, 10)),
            AdapterCommand::SetColor(0, (255, 255, 255, 255)),
        ]);
    });
}
