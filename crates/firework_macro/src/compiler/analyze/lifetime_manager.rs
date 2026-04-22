// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use std::collections::HashMap;
use rand::Rng;
use syn::Lifetime;

pub use super::*;

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

    // Явлется ли это ссылкой на спарк (только для shared)
    pub is_spark_ref: bool,
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
        // [WHY RAND] [NO FIX THIS]
        // Внимание, использование rand явлется самым оптимальным решением в архитектуре.
        // Другие методы не работают либо создают поведение которое ломает фреймворк
        //  - Счётчик не работает так как компилятор может кэшировать вызов макроса и
        //    айди начнут налезать друг на друга ломая навигацию
        //  - Сигнатуру функции использовать нельзя потому-что в разных модулях может
        //    быть функция с одной сигнатурой
        //  - Спан использовать нельзя, может быть коллизия
        //  - Получить точное место в файле нельзя в стабильном rust
        // Главное зачем нужен screen_index это создать в коде константу которая поможет
        // надёжно сравнить текущую функцию и функцию которая была вызвана чтобы настроить
        // контекст на Build или Navigate флэш

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
    /// мертва и чтобы не было паники нужно вернуть значение обратно в BSS память со стэка.
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
            // DropSpark не должен быть сгенерирован если is_spark_ref это true, то есть
            // переменная является ссылкой на состояние в shared, а не владением. Генерация
            // возврата не нужна
            if !scope.variables.contains_key(name) && value.is_spark && !value.is_spark_ref {
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

impl<'ast> Analyzer {
    /// Этот метод используется в break и continue чтобы найти последнюю область
    /// видимости которая явлется циклом, label нужен для циклов с именем, принимает
    /// опциональный Lifetime от syn, а возвращает область видимости которая была
    /// найдена в стэке
    pub(crate) fn get_target_scope(&mut self, label: &Option<Lifetime>) -> Scope {
        // Получение последней области видимости в стэке
        let target_scope = if let Some(label_break) = label {
            // Имя цикла который нужно остановить
            let label_name = label_break.ident.to_string();

            // Поиск цикла с таким именем по стэку областей видимости
            self.lifetime_manager.old_scope.iter()
                .rev()
                .find(|s| s.label.as_ref() == Some(&label_name))
                .cloned()
                .unwrap_or_else(|| Scope::new())
        } else {
            // Если нет имени цикла в break {'имя} <- вот тут
            self.lifetime_manager.old_scope.iter()
                .rev()
                .find(|s| s.is_cycle)
                .cloned()
                .unwrap_or_else(|| Scope::new())
        };

        target_scope
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::codegen::actions::{FireworkAction, FireworkStatement};
    use proc_macro2::Span;

    #[test]
    fn test_lifetime_checker_update_scope_drops_single_spark() {
        let mut lifetime_manager = LifetimeManager::new();
        
        let old_scope = Scope::new();
        
        let spark_var = Variable {
            variable_type: "i32".to_string(),
            is_spark: true,
            is_mut: false,
            spark_id: 1,
            is_spark_ref: false,
        };
        lifetime_manager.scope.variables.insert("test_spark".to_string(), spark_var);
        
        let base_stmt = FireworkStatement {
            action: FireworkAction::DefaultCode,
            is_reactive_block: false,
            index: 0,
            screen_name: String::from(""),
            string: String::from("original"),
            parent_widget_id: None,
            reactive_loop: false,
            depth: 0,
            screen_index: 0,
            span: Span::call_site(),
        };
        
        let result = lifetime_manager.update_scope(old_scope, false, &base_stmt);
        
        assert_eq!(result.len(), 1);
        
        match &result[0].action {
            FireworkAction::DropSpark { name, id } => {
                assert_eq!(name, "test_spark");
                assert_eq!(*id, 1);
            }
            _ => panic!("Expected DropSpark action"),
        }
        
        assert_eq!(result[0].string, "");
    }

    #[test]
    fn test_lifetime_checker_update_scope_drops_multiple_sparks() {
        let mut lifetime_manager = LifetimeManager::new();
        
        let old_scope = Scope::new();
        
        let spark_var1 = Variable {
            variable_type: "i32".to_string(),
            is_spark: true,
            is_mut: false,
            spark_id: 1,
            is_spark_ref: false,
        };
        
        let spark_var2 = Variable {
            variable_type: "String".to_string(),
            is_spark: true,
            is_mut: true,
            spark_id: 2,
            is_spark_ref: false,
        };

        let spark_var3 = Variable {
            variable_type: "bool".to_string(),
            is_spark: true,
            is_mut: false,
            spark_id: 3,
            is_spark_ref: false,
        };
        
        lifetime_manager.scope.variables.insert("spark1".to_string(), spark_var1);
        lifetime_manager.scope.variables.insert("spark2".to_string(), spark_var2);
        lifetime_manager.scope.variables.insert("spark3".to_string(), spark_var3);
        
        let base_stmt = FireworkStatement {
            action: FireworkAction::DefaultCode,
            is_reactive_block: false,
            index: 0,
            screen_name: String::from(""),
            string: String::from(""),
            parent_widget_id: None,
            reactive_loop: false,
            depth: 0,
            screen_index: 0,
            span: Span::call_site(),
        };
        
        let result = lifetime_manager.update_scope(old_scope, false, &base_stmt);
        
        assert_eq!(result.len(), 3);
        
        let mut spark_ids: Vec<usize> = result.iter().map(|stmt| {
            match &stmt.action {
                FireworkAction::DropSpark { id, .. } => *id,
                _ => panic!("Expected DropSpark action"),
            }
        }).collect();
        
        spark_ids.sort();
        assert_eq!(spark_ids, vec![1, 2, 3]);
    }

    #[test]
    fn test_lifetime_checker_update_scope_ignores_non_spark_variables() {
        let mut lifetime_manager = LifetimeManager::new();
        
        let old_scope = Scope::new();
        
        let normal_var = Variable {
            variable_type: "i32".to_string(),
            is_spark: false,
            is_mut: true,
            spark_id: 0,
            is_spark_ref: false,
        };
        
        lifetime_manager.scope.variables.insert("normal".to_string(), normal_var);
        
        let base_stmt = FireworkStatement {
            action: FireworkAction::DefaultCode,
            is_reactive_block: false,
            index: 0,
            screen_name: String::from(""),
            string: String::from(""),
            parent_widget_id: None,
            reactive_loop: false,
            depth: 0,
            screen_index: 0,
            span: Span::call_site(),
        };
        
        let result = lifetime_manager.update_scope(old_scope, false, &base_stmt);
        
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_lifetime_checker_update_scope_mixed_variables() {
        let mut lifetime_manager = LifetimeManager::new();
        
        let old_scope = Scope::new();
        
        let spark_var = Variable {
            variable_type: "i32".to_string(),
            is_spark: true,
            is_mut: false,
            spark_id: 100,
            is_spark_ref: false,
        };
        
        let normal_var = Variable {
            variable_type: "f64".to_string(),
            is_spark: false,
            is_mut: false,
            spark_id: 0,
            is_spark_ref: false,
        };
        
        lifetime_manager.scope.variables.insert("spark".to_string(), spark_var);
        lifetime_manager.scope.variables.insert("normal".to_string(), normal_var);
        
        let base_stmt = FireworkStatement {
            action: FireworkAction::DefaultCode,
            is_reactive_block: false,
            index: 0,
            screen_name: String::from(""),
            string: String::from(""),
            parent_widget_id: None,
            reactive_loop: false,
            depth: 0,
            screen_index: 0,
            span: Span::call_site(),
        };
        
        let result = lifetime_manager.update_scope(old_scope, false, &base_stmt);
        
        assert_eq!(result.len(), 1);
        
        match &result[0].action {
            FireworkAction::DropSpark { name, id } => {
                assert_eq!(name, "spark");
                assert_eq!(*id, 100);
            }
            _ => panic!("Expected DropSpark action"),
        }
    }

    #[test]
    fn test_lifetime_checker_update_scope_preserves_existing_variables() {
        let mut lifetime_manager = LifetimeManager::new();
        
        let mut old_scope = Scope::new();
        let existing_var = Variable {
            variable_type: "i32".to_string(),
            is_spark: true,
            is_mut: false,
            spark_id: 999,
            is_spark_ref: false,
        };
        old_scope.variables.insert("existing".to_string(), existing_var);
        
        let spark_var = Variable {
            variable_type: "String".to_string(),
            is_spark: true,
            is_mut: false,
            spark_id: 1,
            is_spark_ref: false,
        };
        lifetime_manager.scope.variables.insert("new_spark".to_string(), spark_var);
        
        let base_stmt = FireworkStatement {
            action: FireworkAction::DefaultCode,
            is_reactive_block: false,
            index: 0,
            screen_name: String::from(""),
            string: String::from(""),
            parent_widget_id: None,
            reactive_loop: false,
            depth: 0,
            screen_index: 0,
            span: Span::call_site(),
        };
        
        let result = lifetime_manager.update_scope(old_scope, false, &base_stmt);
        
        assert_eq!(result.len(), 1);
        
        match &result[0].action {
            FireworkAction::DropSpark { name, id } => {
                assert_eq!(name, "new_spark");
                assert_eq!(*id, 1);
            }
            _ => panic!("Expected DropSpark action"),
        }
    }

    #[test]
    fn test_lifetime_checker_update_scope_with_set_scope_true() {
        let mut lifetime_manager = LifetimeManager::new();
        
        let mut new_scope = Scope::new();
        let test_var = Variable {
            variable_type: "bool".to_string(),
            is_spark: false,
            is_mut: false,
            spark_id: 0,
            is_spark_ref: false,
        };
        new_scope.variables.insert("existing_var".to_string(), test_var);
        new_scope.depth = 5;
        new_scope.is_cycle = true;
        new_scope.label = Some("loop_label".to_string());
        new_scope.screen_index = 12345;
        
        let base_stmt = FireworkStatement {
            action: FireworkAction::DefaultCode,
            is_reactive_block: false,
            index: 0,
            screen_name: String::from(""),
            string: String::from(""),
            parent_widget_id: None,
            reactive_loop: false,
            depth: 0,
            screen_index: 0,
            span: Span::call_site(),
        }; 
        
        let result = lifetime_manager.update_scope(new_scope, true, &base_stmt);
        
        assert_eq!(lifetime_manager.scope.depth, 5);
        assert_eq!(lifetime_manager.scope.is_cycle, true);
        assert_eq!(lifetime_manager.scope.label, Some("loop_label".to_string()));
        assert_eq!(lifetime_manager.scope.screen_index, 12345);
        assert!(lifetime_manager.scope.variables.contains_key("existing_var"));
        
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_lifetime_checker_update_scope_no_drops_when_variables_match() {
        let mut lifetime_manager = LifetimeManager::new();
        
        let mut old_scope = Scope::new();
        let spark_var = Variable {
            variable_type: "i32".to_string(),
            is_spark: true,
            is_mut: false,
            spark_id: 1,
            is_spark_ref: false,
        };
        old_scope.variables.insert("same_spark".to_string(), spark_var);
        
        lifetime_manager.scope.variables.insert("same_spark".to_string(), Variable {
            variable_type: "i32".to_string(),
            is_spark: true,
            is_mut: false,
            spark_id: 1,
            is_spark_ref: false,
        });
        
        let base_stmt = FireworkStatement {
            action: FireworkAction::DefaultCode,
            is_reactive_block: false,
            index: 0,
            screen_name: String::from(""),
            string: String::from(""),
            parent_widget_id: None,
            reactive_loop: false,
            depth: 0,
            screen_index: 0,
            span: Span::call_site(),
        };
        
        let result = lifetime_manager.update_scope(old_scope, false, &base_stmt);
        
        assert_eq!(result.len(), 0);
    }
}
