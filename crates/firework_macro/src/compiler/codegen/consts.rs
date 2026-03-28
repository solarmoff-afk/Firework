// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub const SCREEN_HEADER: &str = "

";

pub const CHECK_EVENT: &str = "
\tlet mut _fwc_event = firework::LifeCycle::Zero;
\tif _fwc_id == firework::get_focus() {
\t\t_fwc_event = firework::LifeCycle::Event;
\t} else {
\t\tif _fwc_build {
\t\t\tfirework::adapter_command(firework::AdapterCommand::RemoveAll);
\t\t\t_fwc_event = firework::LifeCycle::Build;
\t\t} else {
\t\t\tfirework::adapter_command(firework::AdapterCommand::RemoveAll);
\t\t\t_fwc_event = firework::LifeCycle::Navigate;
\t\t}
\t}
";

pub const SET_FOCUS: &str = "
\tfirework::set_focus(_fwc_id);
";