// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

/// Адаптер рендера не установлен. Сгенерированный код попытался отправить команду в адаптер,
/// но он не установлен.
pub const RENDER_ADAPTER_MISSING_ERROR: &str = "\
error[FRE001]: render adapter not set
   = note: generated code attempted to send a command to the render adapter, but no adapter is installed
   = help: remove the \"no-default-render\" feature flag from your Cargo.toml
   = help: or explicitly set an adapter using `firework_ui::run_with_adapter(adapter, initial_screen)`
   = help: adapter signature: `fn name(command: AdapterCommand) -> AdapterResult {}`
   = note: for more information, see: [WORK IN PROGRESS]
";
