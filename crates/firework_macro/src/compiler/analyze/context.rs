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

    // Буферный стейтемент куда записываются данные и он потом пушится в IR
    pub statement: FireworkStatement,

    // Промежуточное представление, строки кода с добавлением семантической метки
    pub ir: FireworkIR,

    // Стэк спарков который содержит все спарки в верхних if/match/effect/for/while чтобы
    // определить от каких спарков зависит выполнение этого кода, используется в условных
    // виджетах чтобы деактивировать нужный бит в случае обновления спарка от которого зависит 
    // условный рендеринг
    pub spark_stack: Vec<(String, usize)>,

    // Счётчики чтобы генерировать названия полей глобальной структуры экрана
    pub widget_counter: usize,

    // Количество условных виджетов (в if/match)
    pub maybe_widgets_counter: usize,
    pub spark_counter: usize,

    // Определяет первый ли это лайаут в дереве
    pub layouts_count: usize,

    pub flags: CompileFlags,

    // При добавлении функции сюда долбавляется 1, это нужно чтобы определить явлется ли это
    // первой функцией чтобы не генерировать поле лишний раз в компиляции shared юнита
    pub functions_count: u16,

    // При входе в условие или match это поле помечается как true, а при выходе как false.
    // Если при декларации виджета это поле true то виджет становится условным
    pub is_maybe: bool,

    // Локальная карта айди спарка -> айди условных виджетов которые создаются в блоке который
    // зависит от этого спарка
    pub spark_widget_map: HashMap<usize, Vec<usize>>,

    // Вектор виджетов которые были созданы, используется для динамических списков чтобы
    // снапшотить вектор при входе и забрать значение после чтобы понять нужно ли генерировать
    // специальный код и если да то для каких виджетов. Содержит айди виджета (не условного,
    // а основной счётчик айди для полей в структуре)
    pub microruntime_widgets: MicroruntimeWidgets,

    // Вложенность цикла
    pub cycle_depth: usize,

    // Стэк хуков на реактивные блоки. Если в одном из дочерних блоков есть виджет то идёт
    // проход по всему стэку, из кортежа берётся (айди элемента в IR, Span в снапшоте) и
    // этот реактивный блок помечается как часть декларации UI
    pub reactive_block_stack: Vec<IrHook>,

    // Какой компонент сейчас реализуется
    pub now_component: Option<String>,
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
            spark_stack: Vec::new(),
            
            // Счётчики
            widget_counter: 0,
            maybe_widgets_counter: 0,
            spark_counter: 0,
            layouts_count: 0,

            flags: CompileFlags::new(),
            functions_count: 0,
            is_maybe: false,
            spark_widget_map: HashMap::new(),
            microruntime_widgets: MicroruntimeWidgets::new(),
            cycle_depth: 0,
            reactive_block_stack: Vec::new(),
            now_component: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MicroruntimeWidgets {
    // Количество вложенных циклов на данный момент
    pub count: usize,
   
    // Виджеты в этом цикле
    pub widgets: Vec<usize>,

    // Был ли обновлён счётчик за в этой области видимости
    pub is_dirty: bool,

    // Есть ли виджеты вообще тут либо в дочерних циклах
    pub has_widgets: bool, 
}

impl MicroruntimeWidgets {
    pub fn new() -> Self {
        Self {
            count: 0,
            widgets: Vec::new(),
            is_dirty: false,
            has_widgets: false, 
        }
    }
}
