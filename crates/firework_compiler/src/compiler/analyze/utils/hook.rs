// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

#![allow(dead_code)]

use super::super::*;

/// Хук нужен для того чтобы сохранить координаты записи в IR чтобы вернуться туда через
/// время и прочитать или изменить
#[derive(Debug, Clone)]
pub struct IrHook {
    // Позиция в векторе стейтементов
    pub index: usize,

    // Ключ в снапшоте (карте спан -> вектор стейтементов). Хранит конкретный ключ по которому
    // нужно найти вектор (первое значение) и индекс хука в векторе стейтементов
    pub key: (SpanKey, usize),
}

impl IrHook {
    pub fn new(index: usize, span: SpanKey, index_in_snapshot: usize) -> Self {
        Self {
            index,
            key: (span, index_in_snapshot),
        }
    }

    pub fn from_key(span: SpanKey, index_in_snapshot: usize) -> Self {
        Self {
            index: 0,
            key: (span, index_in_snapshot),
        }
    }

    pub fn null() -> Self {
        Self {
            index: usize::MAX,
            key: (SpanKey::null(), usize::MAX),
        }
    }
}

impl Analyzer {
    pub(crate) fn get_statement_from_hook(&mut self, hook: IrHook) -> &mut FireworkStatement {
        match self
            .context
            .ir
            .get_statement_by_spankey(hook.key.0, hook.key.1)
        {
            Some(statement) => statement,
            None => panic!("IE:3"),
        }
    }

    /// Создаёт хук из последнего элемента IR
    pub(crate) fn get_hook(&self) -> Option<IrHook> {
        if let Some(span) = self.context.ir.get_current_span()
            && let Some(count) = self.context.ir.get_current_statements_count()
        {
            // SAFETY: Если этот код выполняется то span и count есть, а значит
            // есть и стейтементы
            let local_index = count.checked_sub(1).expect("BLOCK::IE:4");

            return Some(IrHook::new(local_index, span.clone(), local_index));
        }

        None
    }
}
