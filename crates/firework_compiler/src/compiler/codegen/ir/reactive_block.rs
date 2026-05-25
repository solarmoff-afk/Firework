// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

/// Какой это конкретно реактивный блок
#[derive(Debug, Clone, PartialEq)]
pub enum FireworkReactiveBlock {
    // Условие
    ReactiveIf,

    // Цикл for
    ReactiveFor,

    // Цикл while
    ReactiveWhile,

    // Match
    ReactiveMatch,

    // Эффект, реактивный блок без условия
    Effect,
}
