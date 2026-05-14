// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use firework_adapter::TestCommand;
use firework_ui::{AdapterCommand, AdapterResult, null_adapter, run_with_adapter};
use std::cell::RefCell;

thread_local! {
    static COMMANDS: RefCell<Vec<TestCommand>> = RefCell::new(Vec::new());
}

pub struct TestHarness;

impl TestHarness {
    pub fn adapter_handler(command: AdapterCommand) -> AdapterResult {
        if !matches!(command, AdapterCommand::RunLoop { .. }) {
            let test_cmd = TestCommand::from(command);
            println!("Command: {:#?}", test_cmd);
            COMMANDS.with(|cmds| cmds.borrow_mut().push(test_cmd));
        }

        null_adapter(command)
    }

    pub fn run(ui_fn: fn()) -> Vec<TestCommand> {
        COMMANDS.with(|cmds| cmds.borrow_mut().clear());
        run_with_adapter(Self::adapter_handler, ui_fn);

        COMMANDS.with(|cmds| cmds.borrow().clone())
    }
}
