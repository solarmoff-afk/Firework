// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use proc_macro2::Span;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use super::FireworkStatement;

#[derive(Debug, Clone, Eq)]
pub struct SpanKey {
    inner: String,
}

impl SpanKey {
    #[must_use]
    pub fn from_span(span: Span) -> Self {
        // Так как спан не реалтзует хэш можно использовать хитрость и использовать Debug
        // вывод черех форматирование чтобы получить строку которую можно хэшировать
        // в хэшмапе
        Self {
            inner: format!("{:?}", span),
        }
    }

    pub fn null() -> Self {
        Self {
            inner: "null".to_string(),
        }
    }
}

impl PartialEq for SpanKey {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Hash for SpanKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub statements: HashMap<SpanKey, Vec<FireworkStatement>>,
    pub order: Vec<SpanKey>,
}

impl Snapshot {
    #[must_use]
    pub fn new() -> Self {
        Self {
            statements: HashMap::new(),
            order: Vec::new(),
        }
    }
}

impl Default for Snapshot {
    fn default() -> Self {
        Self::new()
    }
}
