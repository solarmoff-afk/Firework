// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

mod common;

use crate::common::TestHarness;
use firework_adapter::TestCommand;
use firework_ui::{BuildContext, Prop, component, ui};

type AdapterCommand = TestCommand;

component! {
    pub struct BasicComponent {}

    impl BasicComponent {
        pub fn new() -> Self {
            Self {}
        }

        pub fn flash(&mut self, _context: BuildContext) {
            rect! {
                position: (10, 10),
                color: (255, 0, 0),
            }
        }
    }
}

#[ui]
fn test_ui_basic_component_screen() {
    component! {
        target: BasicComponent,
    }
}

#[test]
fn test_ui_basic_component() {
    let commands = TestHarness::run(test_ui_basic_component_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetColor(0, (255, 255, 255, 0)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (10, 10)),
            AdapterCommand::SetColor(0, (255, 0, 0, 255)),
            AdapterCommand::SharedVCanvas(0, 0),
            AdapterCommand::SetZ(0, 0),
            AdapterCommand::SetZ(0, 0),
            AdapterCommand::SetZ(0, 3),
        ]
    );
}

component! {
    pub struct PropComponent {
        pub color: Prop<(u8, u8, u8)>,
        pub x_pos: Prop<i32>,
    }

    impl PropComponent {
        pub fn new() -> Self {
            Self {
                color: None,
                x_pos: None,
            }
        }

        pub fn flash(&mut self, _context: BuildContext) {
            rect! {
                position: (self.x_pos.unwrap_or(0), 20),
                color: self.color.unwrap_or((255, 255, 255)),
            }
        }
    }
}

#[ui]
fn test_ui_prop_component_screen() {
    component! {
        target: PropComponent,
        color: (0, 255, 0),
        x_pos: 100,
    }
}

#[test]
fn test_ui_prop_component() {
    let commands = TestHarness::run(test_ui_prop_component_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetColor(0, (255, 255, 255, 0)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (100, 20)),
            AdapterCommand::SetColor(0, (0, 255, 0, 255)),
            AdapterCommand::SharedVCanvas(0, 0),
            AdapterCommand::SetZ(0, 0),
            AdapterCommand::SetZ(0, 0),
            AdapterCommand::SetZ(0, 3),
        ]
    );
}

component! {
    pub struct StateComponent {}

    impl StateComponent {
        pub fn new() -> Self {
            Self {}
        }

        pub fn flash(&mut self, _context: BuildContext) {
            let mut internal_width = spark!(50);

            rect! {
                position: (0, 0),
                size: (internal_width, 50),
                color: (0, 0, 255),
            }

            if internal_width == 50 {
                internal_width = 150;
            }
        }
    }
}

#[ui]
fn test_ui_state_component_screen() {
    component! { target: StateComponent, }
}

#[test]
fn test_ui_state_component() {
    let commands = TestHarness::run(test_ui_state_component_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetColor(0, (255, 255, 255, 0)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (0, 0)),
            AdapterCommand::SetSize(0, (50, 50)),
            AdapterCommand::SetColor(0, (0, 0, 255, 255)),
            AdapterCommand::SharedVCanvas(0, 0),
            AdapterCommand::SetSize(0, (150, 50)),
            AdapterCommand::SetZ(0, 0),
            AdapterCommand::SetZ(0, 0),
            AdapterCommand::SetZ(0, 3),
        ]
    );
}

component! {
    pub struct ReactivePropComponent {
        pub offset: Prop<i32>,
    }

    impl ReactivePropComponent {
        pub fn new() -> Self {
            Self { offset: None }
        }

        pub fn flash(&mut self, _context: BuildContext) {
            rect! {
                position: (self.offset.unwrap_or(0), 0),
                color: (255, 255, 0),
            }
        }
    }
}

#[ui]
fn test_ui_reactive_prop_screen() {
    let mut parent_offset = spark!(10);

    component! {
        target: ReactivePropComponent,
        offset: parent_offset,
    }

    if parent_offset == 10 {
        parent_offset = 20;
    }
}

#[test]
fn test_ui_reactive_prop() {
    let commands = TestHarness::run(test_ui_reactive_prop_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetColor(0, (255, 255, 255, 0)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (10, 0)),
            AdapterCommand::SetColor(0, (255, 255, 0, 255)),
            AdapterCommand::SharedVCanvas(0, 0),
            AdapterCommand::SetZ(0, 0),
            AdapterCommand::SetPosition(0, (20, 0)),
            AdapterCommand::SetZ(0, 0),
            AdapterCommand::SetZ(0, 3),
        ]
    );
}

#[ui]
fn test_ui_multiple_components_screen() {
    component! {
        target: PropComponent,
        color: (255, 0, 0),
        x_pos: 10,
    }

    component! {
        target: PropComponent,
        color: (0, 0, 255),
        x_pos: 50,
    }
}

#[test]
fn test_ui_multiple_components() {
    let commands = TestHarness::run(test_ui_multiple_components_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetColor(0, (255, 255, 255, 0)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (10, 20)),
            AdapterCommand::SetColor(0, (255, 0, 0, 255)),
            AdapterCommand::SharedVCanvas(0, 0),
            AdapterCommand::SetZ(0, 0),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetColor(0, (255, 255, 255, 0)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (50, 20)),
            AdapterCommand::SetColor(0, (0, 0, 255, 255)),
            AdapterCommand::SharedVCanvas(0, 0),
            AdapterCommand::SetZ(0, 0),
            AdapterCommand::SetZ(0, 0),
            AdapterCommand::SetZ(0, 3),
            AdapterCommand::SetZ(0, 4),
            AdapterCommand::SetZ(0, 7),
        ]
    );
}

component! {
    pub struct GenericComponent<T> {
        pub generic_val: Prop<T>,
    }

    impl<T: Default + Copy + Into<i32>> GenericComponent<T> {
        pub fn new() -> Self {
            Self { generic_val: None }
        }

        pub fn flash(&mut self, _context: BuildContext) {
            let val: i32 = self.generic_val.unwrap_or(T::default()).into();

            rect! {
                position: (val, val),
                color: (50, 50, 50),
            }
        }
    }
}

#[ui]
fn test_ui_generic_component_screen() {
    component! {
        target: GenericComponent,
        T: i32,
        generic_val: 42,
    }
}

#[test]
fn test_ui_generic_component() {
    let commands = TestHarness::run(test_ui_generic_component_screen);

    assert_eq!(
        commands,
        vec![
            AdapterCommand::RemoveAll,
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetColor(0, (255, 255, 255, 0)),
            AdapterCommand::NewRect { layout: 1 },
            AdapterCommand::SetHitGroup(0, 65535),
            AdapterCommand::SetPosition(0, (42, 42)),
            AdapterCommand::SetColor(0, (50, 50, 50, 255)),
            AdapterCommand::SharedVCanvas(0, 0),
            AdapterCommand::SetZ(0, 0),
            AdapterCommand::SetZ(0, 0),
            AdapterCommand::SetZ(0, 3),
        ]
    );
}
