// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

#![allow(dead_code)]

pub mod snapshot;
pub mod actions;
pub mod reactive_block;
pub mod widget;
pub mod shared;

use proc_macro2::Span;
use std::collections::HashMap;

pub use snapshot::{Snapshot, SpanKey};
pub use actions::FireworkAction;
pub use shared::SharedData;
pub use widget::{FireworkWidgetField, WidgetDescription};
pub use reactive_block::FireworkReactiveBlock;
pub use shared::FireworkSharedState;

/// Раст команда (statement) записанная анализатором
#[derive(Debug, Clone)]
pub struct FireworkStatement {
    // Семантическая метка которая кратко говорит что делает эта строка, создаёт спарк,
    // обновляет спарк, дропает спарк и так далее
    pub action: FireworkAction,
    
    // Явлется ли это реактивным блоком
    pub is_reactive_block: bool,

    // Текущий индекс (Может быть полезен для дебага, работает через счётчик)
    pub index: usize,

    // Имя экрана к которому относится стейтемент
    pub screen_name: String,

    // Строковое представления собранное из токенов, нужно для инлайна
    pub string: String,

    // К какому виджету принадлежит, это нужно для содержимого замыканий в виджетах чтобы
    // определить какой именно блок кода дёргать зная айди виджета
    pub parent_widget_id: Option<usize>,

    pub reactive_loop: bool,

    pub depth: u16,
    pub screen_index: u128,

    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FireworkIR {
    // Айди элемента в векторе это номер statement
    pub statements: Vec<FireworkStatement>,

    // Карта Спан -> Виртуальный стейтемент
    pub snapshot: Snapshot, 

    // Последний спан который был задан. Используется чтобы разместить 
    pub last_span: Option<SpanKey>,

    // Соотвествие экрана (название функции) и структуры экрана в формате вектора
    // кортежей (Имя поля, тип) для структуры
    pub screen_structs: HashMap<String, Vec<(String, String)>>,

    pub screens: Vec<(
        String,
        String,
        u128, // Id экрана
    )>,

    // Хэшмап для хранения id экрана -> количество спарков
    pub screen_sparks: HashMap<u128, usize>,

    // Хэшмап для хранения id экрана -> условные виджеты
    pub screen_maybe_widgets: HashMap<u128, MaybeWidgets>,

    pub shared: SharedData,
}

/// Структура для хранения состяния условных виджетов
#[derive(Debug, Clone)]
pub struct MaybeWidgets {
    // Сколько всего условных виджетов
    pub count: usize,

    // Карта айди спарка -> айди условных виджетов которые создаются в блоке который зависит
    // от этого спарка
    pub spark_widget_map: HashMap<usize, Vec<usize>>,
}

impl MaybeWidgets {
    pub fn new() -> Self {
        Self {
            count: 0,
            spark_widget_map: HashMap::new(),
        }
    }
}

impl FireworkIR {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
            snapshot: Snapshot::new(), 
            last_span: None,
            screen_structs: HashMap::new(),
            screens: Vec::new(),
            screen_sparks: HashMap::new(),
            screen_maybe_widgets: HashMap::new(),
            shared: SharedData::new()
        }
    }
   
    /// Добавляет виртуальный стейтемент в IR, использует текущий спан который устанавливается
    /// через метод set_span как ключ в снапшоте чтобы записать туда виртуальный стейтемент
    pub fn push(&mut self, stmt: FireworkStatement) {
        let span_key = SpanKey::from_span(stmt.span);
        self.last_span = Some(span_key.clone());
        
        // Теперь просто вставляем, без Vec
        self.snapshot
            .statements
            .entry(span_key.clone())
            .or_insert_with(Vec::new)
            .push(stmt.clone());
        
        self.statements.push(stmt);
        
        if !self.snapshot.order.contains(&span_key) {
            self.snapshot.order.push(span_key);
        }
    }
    
    /// Устанавливает спан по которому будут добавлены виртуальные стейтементы через
    /// push после этого вызова
    pub fn set_span(&mut self, span: Span) {
        self.last_span = Some(SpanKey::from_span(span));
    }

    /// Получение текущего спан ключа (Спан ключ это строка которая получена из Span)
    pub fn get_current_span(&self) -> Option<&SpanKey> {
        self.last_span.as_ref()
    }

    pub fn get_current_statements(&self) -> Option<&Vec<FireworkStatement>> {
        if let Some(span_key) = &self.last_span {
            self.snapshot.statements.get(span_key)
        } else {
            None
        }
    }

    pub fn get_current_statements_mut(&mut self) -> Option<&mut Vec<FireworkStatement>> {
        if let Some(span_key) = &self.last_span {
            self.snapshot.statements.get_mut(span_key)
        } else {
            None
        }
    }

    pub fn get_statements_by_span(&self, span: Span) -> Option<&Vec<FireworkStatement>> {
        let key = SpanKey::from_span(span);
        self.snapshot.statements.get(&key)
    }

    pub fn get_statements_by_span_mut(&mut self, span: Span) -> Option<&mut Vec<FireworkStatement>> {
        let key = SpanKey::from_span(span);
        self.snapshot.statements.get_mut(&key)
    }
}
