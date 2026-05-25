// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub mod cycle_checker;

mod warnings;

use cycle_checker::CycleChecker;
use proc_macro2::{Span, TokenStream};
use rand::Rng;
use std::collections::HashMap;

/// Линтер нужен чтобы анализировать код пользователя на наличие поведения которое не явлется
/// ошибкой, но может работать неожиданно для пользователя. Линтер должен дать подробное
/// предупреждение
pub struct FireworkLinter {
    // Вектор предупреждений, так как proc-macro не поддерживает предупреждения нативно
    // используется хак с deprecated (Так как ошибки это слишком жёстко), все предупреждения
    // собираются как TokenStream и добавляются к сгенерированному коду с оригинальным
    // спаном
    pub warnings: Vec<TokenStream>,

    // Карта айди спарка -> (Имя, строка кода). Используется для составления предупреждений
    // которые связаны с реактиностью и состоянием
    pub nodes_map: HashMap<usize, (String, String)>,

    // Чекеры

    // Чекер циклических зависимостей (a -> b -> a) в графе зависимостей. Создаёт граф
    // зависимостей и ищет циклические ссылки
    cycle_checker: CycleChecker,

    // Так как для предупреждений используется вставка в итоговый код нужно использовать
    // случайное число для формирования функций FW001_{counter}, FW001_0, FW001_1 и так
    // далее
    counter: u64,
}

impl FireworkLinter {
    pub fn new() -> Self {
        Self {
            warnings: Vec::new(),
            nodes_map: HashMap::new(),
            cycle_checker: CycleChecker::new(),
            counter: 0,
        }
    }

    /// Сброс при выходе из функции чтобы сбросить старый локальный контекст
    pub fn reset(&mut self) {
        self.cycle_checker.reset();
        self.nodes_map.clear();
        self.reset_counter();
    }

    /// Вызывается при добавлении спарка, передаётся его айди, имя и строка кода
    pub fn add_spark(&mut self, id: usize, name: String, code: String) {
        self.cycle_checker.add_spark(id);
        self.nodes_map.insert(id, (name, code));
    }

    /// Вызывается при создании computed выражения, принимает два айди и спан, если
    /// есть циклическая ссылка то будет предупреждение FW001
    pub fn depend_spark(&mut self, id_parent: usize, id_child: usize, span: Span) {
        if let Some(cycle_path) = self.cycle_checker.depend(id_parent, id_child) {
            self.reset_counter();

            let warning = self.generate_cycle_warning(id_parent, id_child, &cycle_path, span);

            self.warnings.push(warning);
        }
    }

    /// Сброс счётчика, новое случайное число
    fn reset_counter(&mut self) {
        let mut range = rand::thread_rng();
        self.counter = range.gen_range(0..=u64::MAX);
    }
}
