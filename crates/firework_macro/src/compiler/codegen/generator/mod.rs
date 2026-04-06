// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

mod base;
mod static_gen;
mod reactive;

use std::collections::HashMap;
use rand::Rng;
use reactive::bitmask_gen::get_spark_mask;

use super::actions::{FireworkIR, FireworkAction};
use super::consts::*;

// NOTE: Дополнительные методы реализованы в base.rs
pub struct CodeGen {
    pub ir: FireworkIR,

    // Хэш мап для хранения результатов кодогенерации для каждого экрана
    screen_map: HashMap<String, (String, u64)>,

    // Хэш мап айди экрана -> Количество битовых масок
    screen_bitmask_count_map: HashMap<String, u8>,

    // Старое значение флага у FireworkStatement который означает обёрнут ли
    // блок в loop { ... }, если старый флаг был true то нужно сгенерировать
    // выход из цикла, если false то сгенерировать вход в цикл. Анализатор
    // атоматически добавил стейтемент с меткой Terminator который нужен чтобы
    // зафиксировать изменение этого флага
    old_reactive_loop_flag: bool,
}

impl CodeGen {
    pub fn new(ir: FireworkIR) -> Self {
        Self {
            ir,
            screen_map: HashMap::new(),
            screen_bitmask_count_map: HashMap::new(),
            old_reactive_loop_flag: false,
        }
    }

    /// Запустить кодогенерацию
    pub fn run(&mut self) -> String {
        let mut output = String::from("");

        self.inline_items(&mut output);
        self.inline_block_struct(&mut output);
        
        // Определение сколько нужно битовых масок для реактивности каждого экрана
        self.find_mask_counts();

        self.make_screens_body(1);
        self.inline_screens(&mut output); 

        output
    }

    /// Полный инлайн функции экрана
    fn inline_screens(&mut self, output: &mut String) {
        for (screen_name, screen_signature, screen_id) in self.ir.screens.iter() { 
            output.push_str(format!("{} {{\n", screen_signature).as_str());
            
            let struct_name = format!("ApplicationUiBlockStruct{}", screen_id);
            let instance_name = struct_name.to_uppercase();
            
            output.push_str("\t// Phase 1: Init\n\n");
           
            // Проверка является ли это первым вызовом функции, так как на каждый экран
            // (функцию) идёт свой экземпляр то можно проверять по нему 
            output.push_str(static_gen::is_first_call(&screen_name).as_str());
            output.push_str("\tlet mut _fwc_build = false;\n");
           
            // Инициализация если экземпляр ещё не инициализирован
            output.push_str(static_gen::init_instance(&instance_name, screen_name).as_str());

            output.push_str(format!("{}",CHECK_EVENT).as_str());
            
            // Устанавливает фокус на этот экран
            output.push_str(format!("{}", SET_FOCUS).as_str()); 
            
            output.push_str("\n\t// Phase 2: Navigate/Build code\n");
            
            // Добавляем код экрана
            if let Some(screen_code) = self.screen_map.get(screen_name) {
                output.push_str(&screen_code.0);
            } 
            
            output.push_str("}\n\n");
        }
    } 

    /// Сборка содержимого функции экрана из её тела, то есть
    ///
    /// fn screen_func {
    ///  // Тело
    /// }
    fn make_screens_body(&mut self, depth_value: usize) {
        for statement in self.ir.statements.iter() {
            // Текущая глубина, мутабельна так как её нужно изменить при входе в цикл
            // реактивности и при выходе из него
            let mut depth = "\t".repeat(depth_value + statement.scope.depth);
            
            if !self.screen_map.contains_key(&statement.screen_name) {
                // Случайный айди для статического экземпляра и структуры чтобы предотвратить
                // попытку пользователя использовать или изменить эти данные
                let id: u64 = rand::thread_rng().gen_range(0..=u64::MAX); 

                self.screen_map.insert(statement.screen_name.clone(), (String::from(SCREEN_HEADER), id));
            }

            // Имя структуры для которой будет создан статический экземпляр для хранения
            // состояния и скинов виджетов
            let struct_name = format!("ApplicationUiBlockStruct{}", statement.scope.screen_index);
            if let Some(screen_code) = self.screen_map.get_mut(&statement.screen_name) {
                // Получение количества битовых масок для цикла по этому значению
                let mask_count = self.screen_bitmask_count_map.get(&statement.screen_name)
                    .unwrap_or(&0);

                // Чтобы избежать ошибки компиляции из-за borrow checker функции для
                // генерации реактивного цикла статические (не требуют передачи self)
                // этот вызов генерирует битовые маски и вход в реактивный цикл если
                // эта строка находится в UI контексте, а прошлый контекст это pre-ui,
                // а если контекст pre-ui, а прошлый контекст ui то генерирует выход
                // из реактивного цикла. Благодаря Terminator выход из цикла будет всегда
                // даже если после ui контекста нет pre-ui контекста
                CodeGen::check_reactive_loop(
                    self.old_reactive_loop_flag, depth_value, &mut depth, screen_code,
                    &statement, *mask_count,
                );

                // После обработки поле который хранит прошлый флаг получает текущий флаг
                // так как на следующем шаге цикла этот блок выполнится после обработки
                self.old_reactive_loop_flag = statement.reactive_loop;

                match &statement.action {
                    // Создание реактивной переменной
                    FireworkAction::InitialSpark { id, expr_body, name, .. } => {
                        let field_name = format!("spark_{}", id);
                        
                        screen_code.0.push_str(format!(
                            "{}if matches!(_fwc_event, firework::LifeCycle::Build) {{\n", depth,
                        ).as_str());
                        
                        screen_code.0.push_str(&static_gen::set_field(
                            &struct_name,
                            &field_name,
                            &expr_body,
                        ));
                        
                        screen_code.0.push_str(format!("{}}}\n\n", depth).as_str()); 
                        
                        // Снятие владения из структуры
                        let getter = format!("{}_INSTANCE.{}", struct_name, field_name);
                        screen_code.0.push_str(
                            format!("{}let mut {} = unsafe {{ {}.take().unwrap() }};\n",
                                depth, name, getter).as_str());
                    },
                    
                    // Обновление реактивной переменной
                    FireworkAction::UpdateSpark(_, id) => {
                        // Реактивная переменная (спарк) обновилась то нужно изменить бит
                        // который соотвествует этому спарку. Для каждого диапазона спарков
                        // (от 0 до 64) своя битовая маска, поэтому эта строка позволяет
                        // определить в какой маске изменить спарк
                        let mask = get_spark_mask(*id);

                        // Генерация кода изменения бита
                        CodeGen::generate_update_spark(screen_code, *id, mask.into(), &depth);
                    
                        // Всё равно нужен инлайн строки в результат кодогенерации
                        screen_code.0.push_str(format!("{}{}\n", depth, statement.string).as_str());
                    },

                    // Возврат реактивной переменной со стэка обратно в статическую память
                    FireworkAction::DropSpark { name, id } => {
                        let field_name = format!("spark_{}", id);

                        // Генерация возврата владения в BSS
                        // TODO: Могут возникнуть ошибки компиляции на уровне rustc если
                        // пользователь переместит владение, так как возврат владения сделать
                        // будет нельзя (Ибо rustc проверит владение на этой строке). Нужно
                        // добавить магию компилятора в будущем
                        screen_code.0.push_str(&static_gen::set_field(
                            &struct_name,
                            &field_name,
                            &name,
                        ));
                    },

                    // Обработка реактивных блоков. Реактивный блок это if, for, while или
                    // match в условии которых содержится спарк, например:
                    // if spark1 == 1 {
                    //  // Это реактивный блок
                    // }
                    FireworkAction::ReactiveBlock(block_type, sparks) => {
                        // Первое, условие по которому будет срабатывать реактивный блок
                        screen_code.0.push_str(format!("{}if ", depth).as_str());

                        // Второе, заполнение реактивного блока. Сюда добавляется проверка
                        // на наличие активного бита в битовых масках где находятся спарки
                        // которые используются в условиях
                        for (_, id) in sparks.iter() {
                            CodeGen::generate_check_spark_bit(screen_code, *id);
                            screen_code.0.push_str(" ||");
                        }

                        // Хак для упрощения кодогенератора, только здесь false чтобы
                        // это условие никогда не выполнилось. Это третье
                        screen_code.0.push_str(" false {\n");

                        // Сам реактивный блок в сгенерированном условии
                        screen_code.0.push_str(format!("{}{}\n", depth, statement.string).as_str()); 
                    },

                    // Закрытие реактивного блока
                    FireworkAction::ReactiveBlockTerminator => {
                        // Закрытие условия
                        screen_code.0.push_str(format!("{}}}\n", depth).as_str());

                        // Закрытие проверки зависимостей. Это не должно стать причиной
                        // ошибок компиляции так как ReactiveBlockTerminator есть только
                        // у строк с закрывающей скобкой для реактивных блоков, а реактивный
                        // блок всегда содержит спарк значит для него генерируется условие
                        // с проверкой которое не закрывается и здесь оно может закрыться
                        screen_code.0.push_str(format!("{}}}\n", depth).as_str());

                    }

                    // Терминатор нужен только для проверки флага выше, никакой код он не
                    // генерирует. Он просто означант конец функции экрана
                    FireworkAction::Terminator => {},

                    // Другой случай который либо не реализован, либо DefaultCode (код без
                    // семантической метки)
                    _ => {
                        // Делаем инлайн изначальной строки только если у нас нет специальной логики для
                        // этого действия из FireworkAction
                        screen_code.0.push_str(format!("{}{}\n", depth, statement.string).as_str());
                    },
                };
            }
        }
    }
}
