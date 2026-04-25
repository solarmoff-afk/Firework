// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

// TODO: Добавить какую-то логику
#[allow(unused)]
pub const SCREEN_HEADER: &str = "

";

/// Константа для определения ивента
pub const CHECK_EVENT: &str = "
\tif _fwc_id == firework_ui::get_focus_id() && !_fwc_build {
\t\t_fwc_event = firework_ui::LifeCycle::Event;
\t} else {
\t\tif _fwc_build {
\t\t\tfirework_ui::adapter_command(firework_ui::AdapterCommand::RemoveAll);
\t\t\t_fwc_event = firework_ui::LifeCycle::Build;
\t\t} else {
\t\t\tfirework_ui::adapter_command(firework_ui::AdapterCommand::RemoveAll);
\t\t\t_fwc_event = firework_ui::LifeCycle::Navigate;
\t\t}
\t}
";

/// Константа для того чтобы установить фокус на экран, использует его айди, необходимо
/// завести _fwc_id и дать правильное значение из firework::register для правильной
/// работы
pub const SET_FOCUS: &str = "
\tfirework_ui::set_focus_id(_fwc_id);\n
";

pub const CHECK_NAVIGATE: &str = " firework_ui::tiny_matches!(_fwc_event, firework_ui::LifeCycle::Navigate) || firework_ui::tiny_matches!(_fwc_event, firework_ui::LifeCycle::Build) ";

pub const CHECK_EVENT_INCODE: &str = " firework_ui::tiny_matches!(_fwc_event, firework_ui::LifeCycle::Event) ";
