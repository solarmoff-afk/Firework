// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub use super::*;

pub struct AnalyzeContext {
    // Ошибки компиляции, они накапливаются весь парсинг чтобы по завершению анализа
    // вывести их в терминал. Подробнее про сообщения ошибок можно узнать в файле
    // firework_macro/src/compiler/errors.rs. Все ошибки начинаются с FE, то есть
    // Firework Error и заканчиваются числом из трёх цифр, это номер ошибки. Пример:
    // FE001, FE004
    pub errors: Vec<Error>,
    
    // Выходные токены
    pub output: TokenStream,

    pub statement: FireworkStatement,

    // Промежуточное представление, строки кода с добавлением семантической метки
    pub ir: FireworkIR,

    // Счётчики чтобы генерировать названия полей глобальной структуры экрана
    pub widget_counter: usize,
    pub spark_counter: usize,

    // Определяет первый ли это лайаут в дереве
    pub layouts_count: usize,
}

impl AnalyzeContext {
    pub fn new() -> Self {
        Self {
            // При старте нет ошибок
            errors: Vec::new(),
            
            output: TokenStream::new(),
            
            statement: FireworkStatement {
                action: FireworkAction::DefaultCode,
                is_reactive_block: false,
                index: 0,
                screen_name: String::from(""),
                string: String::from(""),
                parent_widget_id: None,
                reactive_loop: false,
                depth: 0,
                screen_index: 0,
                
                // Указаывает на место макроса по умолчанию, в визиторе будет изменён на
                // span конкретного стейтемента
                span: Span::call_site(),
            },
            
            ir: FireworkIR::new(),
            
            // Счётчики
            widget_counter: 0,
            spark_counter: 0,
            layouts_count: 0,
        }
    }
}
