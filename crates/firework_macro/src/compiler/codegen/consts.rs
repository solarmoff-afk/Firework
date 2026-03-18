// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub const SCREEN_HEADER: &str = "
\t// Если рантайм фреймворка не инициализирован то нужна паника
\tif !firework::is_run() {firework::runtime_error(\"Firework not runned before navigation\")}

\t// Явлется ли это переходом с другого экрана
\tlet _fwc_is_navigate = firework::get_focus() != _FWC_SCREEN_ID;

\tfirework::set_focus(_FWC_SCREEN_ID);


";
