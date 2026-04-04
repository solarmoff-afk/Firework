// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

mod base;
mod static_gen;
mod bitmask_gen;

use std::collections::HashMap;
use rand::Rng;

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
        self.find_mask_counts();

        self.make_screens_body(1);
        self.inline_screens(&mut output);

        for _statement in self.ir.statements.iter() {
            // println!("{:#?}", _statement);
        }

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

    /// Проходится по всем экранам и вычисляет сколько нужно битовых масок для отслеживания
    /// реактивеости, по 64 спарка на одну битовую маску
    fn find_mask_counts(&mut self) {
        for (screen_name, screen_signature, screen_id) in self.ir.screens.iter() {
            // Вычисление количества битовых масок, одна битовая маска это 64 бита
            let spark_count = self.ir.screen_sparks.get(screen_id).unwrap_or(&0usize);

            // Расчёт сколько нужно битовых масок на основе количество спарков
            // 1 -> 1, 19 -> 1, 64 -> 1, 67 -> 2, 98 -> 2, 128 -> 2, 136 -> 3
            self.screen_bitmask_count_map.insert(
                screen_name.to_string(), ((spark_count + 63) / 64) as u8); 
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

                // Проверка, если прошлый стейтемент не был частью реактивного цикла, а
                // текущий элемент таковым явлется то нужно сгенерировать битовую маску
                // и вход в цикл
                if !self.old_reactive_loop_flag && statement.reactive_loop {
                    // Писать логику для сбросв флага после выхода за пределы тела функции
                    // не нужно так как анализатор создаёт терминатор, он автоматически
                    // хранит reactive_loop = false

                    for mask_index in 0u8..*mask_count {
                        screen_code.0.push_str(format!("{}let mut _fwc_bitmask{} = 0u64;\n",
                            depth, mask_index).as_str());
                    }

                    screen_code.0.push_str(format!("{}loop {{\n", depth).as_str());
                    
                    // Так как мы перешли в цикл нужно добавить глубины
                    depth = "\t".repeat(depth_value + 1 + statement.scope.depth);
                }

                // Если наоборот старый флаг говорит что прошлый стейтемент был в цикле, а
                // этот стейтемент не в цикле то нужно закрыть цикл
                if self.old_reactive_loop_flag && !statement.reactive_loop {
                    // Выход из цикла если не было изменений. Цикл будет шагать пока
                    // не будет ситуации когда изменений спарков больше нет, для каждого
                    // бита маски свой спарк
                    // TODO: Реализовать защиту от циклических зависимостей
                    for mask_index in 0u8..*mask_count {
                        screen_code.0.push_str(format!("\n{}if _fwc_bitmask{} == 0 {{ break; }}\n",
                                depth, mask_index).as_str()); 
                    }

                    // Так как это был либо терминатор либо стейтемент который не относится
                    // к циклу реактивности то это завершение и нужно снизить глубину
                    // форматирования
                    depth = "\t".repeat(depth_value + statement.scope.depth);

                    // Выход из цикла
                    screen_code.0.push_str(format!("{}}}\n", depth).as_str());
                }

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
                        screen_code.0.push_str(format!("{}{};\n", depth, bitmask_gen::set_flag(
                                format!("_fwc_bitmask{}", 0)  // Name, TODO: Сделать несколько
                                    .as_str(),                // масок 
                                
                                // Используется айди спарка как бит для отслеживания
                                *id as u8,
                            )
                        ).as_str());
                        
                        // Всё равно нужно проинлайнить код самого присваивания
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
