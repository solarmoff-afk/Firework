// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

#[derive(Debug, Clone, Copy)]
pub enum CompileType {
    Screen,
    Shared,
}

#[derive(Debug, Clone, Copy)]
pub struct CompileFlags {
    pub compile_type: CompileType,
}

impl CompileFlags {
    pub fn new() -> Self {
        Self {
            compile_type: CompileType::Screen,
        }
    }
}
