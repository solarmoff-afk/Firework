// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub use super::super::*;

use crate::compiler::analyze::expr::props::PropsFinder;

impl Analyzer {
    /// Метод обёртка над SparkFinder чтобы быстро найти наличие спарка в выражении
    /// используется в коде чтобы проверить явлется ли блок реактивным и получить вектор
    /// спарков который содержит кортеж (имя, айди)
    pub fn get_sparks(&self, expr: &Expr) -> ExprAnalyzeResult {
        let mut found = Vec::new();

        let mut finder = SparkFinderWithId {
            scope: &self.lifetime_manager.scope,
            found: &mut found,
        };

        finder.visit_expr(expr);

        let mut result = ExprAnalyzeResult::new();
        result.sparks = found;

        if let Some(component_name) = &self.context.now_component
            && let Some(props) = self.context.ir.component_props.get(component_name)
        {
            let mut found = Vec::new();

            let mut finder = PropsFinder {
                props,
                found: &mut found,
            };
            finder.visit_expr(expr);

            result.props.extend(found);
        }

        result
    }
}

/// Результат анализа выражения
#[derive(Debug, Clone)]
pub(crate) struct ExprAnalyzeResult {
    pub sparks: Vec<(String, usize)>,
    pub props: Vec<(String, usize)>,
}

impl ExprAnalyzeResult {
    pub fn new() -> Self {
        Self {
            sparks: Vec::new(),
            props: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.sparks.len()
    }

    /// Удаляет спарк по имени и айди
    pub fn delete_spark(&mut self, spark: (&String, usize)) {
        self.sparks.retain(|(s, _)| s != spark.0);
    }

    pub fn append(&mut self, expr: &mut ExprAnalyzeResult) {
        self.sparks.append(&mut expr.sparks);
    }

    pub fn is_empty(&self) -> bool {
        self.sparks.is_empty()
    }

    pub fn dedup(&mut self) {
        self.sparks.dedup();
    }

    pub fn extend(&mut self, other: &ExprAnalyzeResult) {
        self.sparks.extend(other.sparks.iter().cloned());
    }
}
