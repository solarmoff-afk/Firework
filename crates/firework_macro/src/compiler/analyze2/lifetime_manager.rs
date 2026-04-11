// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use std::collections::HashMap;
use rand::Rng;

use crate::compiler::codegen::actions::FireworkAction;
use crate::compiler::codegen::actions::FireworkStatement;

/// Структура для декларации переменной в структуре области видимости
#[derive(Debug, Clone)]
pub struct Variable {
    // Тип переменной строкой, если не указан то он останется NO_TYPE
    pub variable_type: String,

    // Явлется ли эта переменная реактивной (спарком). Это определяется по налиию
    // макроса spark!() в выражении, но будет ошибка если имя спарка не будет
    // уникальным, если:
    //
    // 1 кейс: Другая переменная затенит спарк (shadowing)
    // 2 кейс: Тип спарка не будет указан при инициализации
    // 3 кейс: Используется несколько спарков в выражении (spark!() + spark!())
    // 
    // Также спарк не определится если не будет в statement::local, поэтому условная
    // инициализация не работает для спарка
    pub is_spark: bool,

    // Явлется ли эта переменная мутабельной
    pub is_mut: bool,

    // Айди спарка (если это спарк, иначе 0) в качестве которого используется счётчик
    // spark_counter
    pub spark_id: usize,
}

/// Текущая область видимости, хранить всю таблицу символов для этой области. Начинается
/// с { и при входе в эту область видимости экземпляр этой структуры будет скопирован
/// чтобы когда произойдёт выход из неё все созданные в ней имена были заменены
/// состояние слепок которого был сделан до входа в область видимости
#[derive(Debug, Clone)]
pub struct Scope {
    pub variables: HashMap<String, Variable>,
    pub screen_index: u128,
    pub depth: u16,
    pub is_cycle: bool,

    // Имя цикла, нужно для синтаксиса break 'label. Если это не цикл то None
    pub label: Option<String>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            // Нет имён на старте
            variables: HashMap::new(),
            screen_index: 0,

            // Первая область видимости имеет нулевую глубину
            depth: 0,

            // Так как начальный Scope не может быть в цикле
            is_cycle: false,
            label: None,
        }
    }

    /// Генерирует случайный айди экрана для надёжности
    pub fn screen_index_generate(&mut self) {
        let mut range = rand::thread_rng();
        self.screen_index = range.gen_range(0..=u128::MAX);
    }
}

pub struct LifetimeManager {
    // Три области видимости которые нужны для реализации лайфтайм детектора
    // Текущая область видимости куда добавляются локальные переменные
    pub scope: Scope,

    // Стэк областей видимости, при вхходе в область видимости делается пуш, при
    // выходе из области видимости pop. Используется для break и continue в менеджере
    // лайфтаймов
    pub old_scope: Vec<Scope>,

    // Дамп область видимости до входа в функцию, нужна для обработки дропа спарков
    // при return
    pub item_scope: Scope,
}

impl LifetimeManager {
    pub fn new() -> Self {
        Self {
            // Три области видимости для лайфтайм менеджера
            scope: Scope::new(),
            old_scope: Vec::new(),
            item_scope: Scope::new(),
        }
    }

    /// Систсема для обработки выхода из области видимости, принимает старую область
    /// видимости (scope) после чего делает сравнение с текущей областью видимости,
    /// локальные переменных которые были созданы в этой области видимости нет в старой
    /// области видимости, алгоритм сравнения найдёт отсуствие переменной и сгенерирует
    /// семантическую метку для IR DropSpark, оно означает что нужно вернуть владение
    /// обратно в BSS так как локальная переменная которая арендовала значение из BSS
    /// мертва и чтобы не было UB нужно вернуть значение обратно в BSS память со стэка.
    /// Так как мы делаем push в IR до обработки следующего statement то в IR сначала
    /// будет возврат в этой же области видимости
    ///
    /// Семантика:
    ///  - self.scope это текущая область видимости
    ///  - scope это старая область видимости которая была до входа в текущую область
    ///    видимости
    pub fn update_scope(
        &mut self,
        scope: Scope,
        set_scope: bool,
        base_statement: &FireworkStatement,
    ) -> Vec<FireworkStatement> {
        let mut statements = Vec::new();
        
        for (name, value) in &self.scope.variables {
            if !scope.variables.contains_key(name) && value.is_spark {
                let mut stmt = base_statement.clone();
                stmt.string = "".to_string();
                stmt.action = FireworkAction::DropSpark {
                    name: name.to_string(),
                    id: value.spark_id,
                };
                statements.push(stmt);
            }
        }

        if set_scope {
            self.scope = scope;
        }
        
        statements
    }
}
