// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use firework_ui::{AdapterCommand, AdapterResult, null_adapter, run_with_adapter};
use std::cell::RefCell;

thread_local! {
    static COMMANDS: RefCell<Vec<AdapterCommand>> = RefCell::new(Vec::new());
}

pub struct TestHarness;

impl TestHarness {
    pub fn adapter_handler(command: AdapterCommand) -> AdapterResult {
        if !matches!(command, AdapterCommand::RunLoop { .. }) {
            println!("Command: {:#?}", command);
            COMMANDS.with(|cmds| cmds.borrow_mut().push(command));
        }

        null_adapter(command)
    }

    pub fn run(ui_fn: fn()) -> Vec<AdapterCommand> {
        COMMANDS.with(|cmds| cmds.borrow_mut().clear());
        run_with_adapter(Self::adapter_handler, ui_fn);

        COMMANDS.with(|cmds| cmds.borrow().clone())
    }
}
