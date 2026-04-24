// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use proc_macro2::Span;
use std::collections::HashMap;

/// Поле разделяемого состояния в shared! {} блоке
/// shared! {
///  state! {
///   my_own: i32 = 10,
///   [name] [type] [init]
///  }
/// }
#[derive(Debug, Clone)]
pub struct FireworkSharedState {
    pub name: String,
    pub spark_type: String,
    pub init: String,
    pub span: Span,
    pub id: usize,
    pub attributes: Vec<String>,
}

/// Структура для хранения информации о shared
#[derive(Debug, Clone)]
pub struct SharedData {
    // Для shared блоков IR содержит вектор состояний которые были объявлены в state! чтобы
    // сгенерировать build функцию для shared блока
    pub state: Vec<FireworkSharedState>,

    // Разделяемое состояние (название) -> Вектор функциональных эффектов
    pub effects: HashMap<String, Vec<String>>,
}

impl SharedData {
    pub fn new() -> Self {
        Self {
            state: Vec::new(),
            effects: HashMap::new(),
        }
    }
}
