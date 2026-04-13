// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use proc_macro2::Span;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use super::actions::FireworkStatement;

#[derive(Debug, Clone, Eq)]
pub struct SpanKey {
    inner: String,
}

impl SpanKey {
    #[must_use]
    pub fn from_span(span: Span) -> Self {
        Self {
            inner: format!("{:?}", span),
        }
    }

    #[must_use]
    pub fn hash_span(span: Span) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        format!("{:?}", span).hash(&mut hasher);
        hasher.finish()
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

    pub fn insert(&mut self, span: Span, stmt: FireworkStatement) {
        let key = SpanKey::from_span(span);
        self.statements
            .entry(key.clone())
            .or_insert_with(Vec::new)
            .push(stmt);

        if !self.order.contains(&key) {
            self.order.push(key);
        }
    }

    #[must_use]
    pub fn get(&self, span: Span) -> Option<&Vec<FireworkStatement>> {
        let key = SpanKey::from_span(span);
        self.statements.get(&key)
    }

    pub fn get_mut(&mut self, span: Span) -> Option<&mut Vec<FireworkStatement>> {
        let key = SpanKey::from_span(span);
        self.statements.get_mut(&key)
    }

    #[must_use]
    pub fn contains(&self, span: Span) -> bool {
        let key = SpanKey::from_span(span);
        self.statements.contains_key(&key)
    }

    pub fn remove(&mut self, span: Span) -> Option<Vec<FireworkStatement>> {
        let key = SpanKey::from_span(span);
        self.order.retain(|k| k != &key);
        self.statements.remove(&key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&SpanKey, &Vec<FireworkStatement>)> {
        self.order
            .iter()
            .filter_map(|key| self.statements.get_key_value(key))
    }
}

impl Default for Snapshot {
    fn default() -> Self {
        Self::new()
    }
}
