// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

#![allow(unused)]

use crate::compiler::codegen::ir::{
    FireworkAction, FireworkIR, FireworkReactiveBlock, FireworkStatement,
};
use crate::compiler::common::logger::{Logger, LoggerColor};

macro_rules! loc {
    ($en:expr, $ru:expr) => {
        if cfg!(feature = "ru") { $ru } else { $en }
    };
}

pub struct HelperFormatter<'a> {
    ir: &'a FireworkIR,
    logger: &'a Logger,
}

impl<'a> HelperFormatter<'a> {
    pub fn new(ir: &'a FireworkIR, logger: &'a Logger) -> Self {
        Self { ir, logger }
    }

    pub fn print_helper_info(&self) {
        let mut was_in_helper = false;

        for stmt in &self.ir.statements {
            if stmt.in_helper {
                if !was_in_helper {
                    self.logger.custom_log(
                        loc!("HELPER", "ПОМОЩНИК"),
                        LoggerColor::Magenta,
                        loc!("--- Code Analysis ---", "--- Разбор кода ---"),
                    );

                    // Объясняем концепцию FlashPass в самом начале
                    self.logger.custom_log(
                        loc!("FLASH PASS", "FLASH PASS"),
                        LoggerColor::Cyan,
                        loc!(
                            "A quick note on how Firework works. Your code doesn't just run once top-to-bottom.\nIt runs multiple times in different 'phases' (Build for the first run, Event for clicks, Reactive for data updates).\nThe compiler's job is to chop this code up and decide which parts run in which phase.",
                            "Пару слов о том, как это работает. Ваш код не просто выполняется один раз сверху вниз.\nОн прокручивается фреймворком несколько раз в разных 'фазах' (Build — первый запуск, Event — клики, Reactive — обновление данных).\nЗадача компилятора — нарезать этот код и указать, что и в какой фазе должно работать."
                        ),
                    );
                    println!();
                }

                self.explain_statement(stmt);
            }
            was_in_helper = stmt.in_helper;
        }

        if was_in_helper {
            println!();
            self.logger.custom_log(
                loc!("HELPER", "ПОМОЩНИК"),
                LoggerColor::Magenta,
                loc!("--- End of analysis ---", "--- Разбор завершен ---"),
            );
            println!();
        }
    }

    fn explain_statement(&self, stmt: &FireworkStatement) {
        let code_line = stmt.string.trim().replace('\n', " ");

        // Для обычного кода (DefaultCode) мы не выводим объяснений.
        // Просто печатаем саму строку, если она не пустая, и идем дальше.
        if let FireworkAction::DefaultCode = &stmt.action {
            if !code_line.is_empty() && code_line != "{" && code_line != "}" {
                self.logger.custom_log(
                    loc!("CODE", "КОД "),
                    LoggerColor::White,
                    &format!(" {}", code_line),
                );
            }
            return;
        }

        // Печатаем код для всех остальных (магических) стейтементов
        if !code_line.is_empty() {
            self.logger.custom_log(
                loc!("CODE", "КОД "),
                LoggerColor::White,
                &format!(" {}", code_line),
            );
        }

        match &stmt.action {
            FireworkAction::InitialSpark {
                name,
                spark_type,
                is_mut,
                ..
            } => {
                let msg = format!(
                    "{}\n  {} `{}`\n  {} `{}`",
                    loc!(
                        "Created a Spark (reactive variable). Under the hood, the compiler allocates static memory for it. If it's `mut`, changing it will trigger UI updates.",
                        "Тут создаётся спарк. По сути, это обычная переменная, но компилятор выделяет под неё место в статике и следит за ней. Если она `mut`, её изменение вызовет перерисовку."
                    ),
                    loc!("Name:", "Имя:"),
                    name,
                    loc!("Type:", "Тип:"),
                    spark_type,
                );
                self.logger
                    .custom_log(loc!("SPARK", "СПАРК"), LoggerColor::Green, &msg);
            }

            FireworkAction::UpdateSpark(name, _, _, _) => {
                let msg = format!(
                    "{} `{}`. {}",
                    loc!("Updating", "Изменяем"),
                    name,
                    loc!(
                        "The compiler will inject a trigger here. When the code hits this line, Firework flags the change and schedules a Reactive phase to update the UI.",
                        "Компилятор вставит сюда невидимый триггер. Как только значение обновится, поднимется флажок, и Firework запустит реактивную фазу, чтобы перерисовать виджеты."
                    )
                );
                self.logger
                    .custom_log(loc!("UPDATE", "ОБНОВЛЕНИЕ"), LoggerColor::Green, &msg);
            }

            FireworkAction::WidgetBlock(desc) => {
                let msg = format!(
                    "{} `{}`.",
                    loc!("Widget declaration:", "Декларация виджета:"),
                    desc.widget_type
                );
                self.logger
                    .custom_log(loc!("WIDGET", "ВИДЖЕТ"), LoggerColor::Blue, &msg);

                if !desc.fields.is_empty() {
                    for (prop_name, field) in &desc.fields {
                        if field.is_fn {
                            let event_msg = format!(
                                "`{}`: {}",
                                prop_name,
                                loc!(
                                    "Event handler. This closure is extracted and will only run during the Event phase (e.g. on click).",
                                    "Обработчик события. Этот код 'вырезается' и будет работать только в фазе Event (например, при клике)."
                                )
                            );
                            self.logger
                                .custom_log("   *", LoggerColor::Magenta, &event_msg);
                        } else if !field.sparks.is_empty() {
                            let deps: Vec<String> =
                                field.sparks.iter().map(|(s, _)| s.clone()).collect();
                            let reactive_msg = format!(
                                "`{}`: {} [{}]",
                                prop_name,
                                loc!(
                                    "Reactive prop. It's wired to these sparks and updates when they change:",
                                    "Реактивный пропс. Он привязан к этим спаркам и обновится при их изменении:"
                                ),
                                deps.join(", ")
                            );
                            self.logger
                                .custom_log("   ~", LoggerColor::Yellow, &reactive_msg);
                        } else {
                            let static_msg = format!(
                                "`{}`: {}",
                                prop_name,
                                loc!(
                                    "Static prop. Evaluated once during Build or Navigate.",
                                    "Статичный пропс. Читается один раз при построении UI."
                                )
                            );
                            self.logger
                                .custom_log("   -", LoggerColor::White, &static_msg);
                        }
                    }
                }
            }

            FireworkAction::ReactiveBlock(block_type, sparks, _) => {
                let kind = match block_type {
                    FireworkReactiveBlock::ReactiveIf => "if",
                    FireworkReactiveBlock::ReactiveFor => "for",
                    FireworkReactiveBlock::ReactiveWhile => "while",
                    FireworkReactiveBlock::ReactiveMatch => "match",
                    FireworkReactiveBlock::Effect => "effect! {}",
                };

                if sparks.is_empty() {
                    let msg = format!(
                        "{} `{}`. {}",
                        loc!("Static block", "Статичный блок"),
                        kind,
                        loc!(
                            "No sparks in the condition, so it runs once. Use `#[raw]` if you want to bypass compiler tracking completely.",
                            "Спарков в условии нет, отработает один раз. Если трекинг компилятором здесь вообще не нужен, используйте атрибут `#[raw]`."
                        )
                    );
                    self.logger
                        .custom_log(loc!("BLOCK", "БЛОК"), LoggerColor::White, &msg);
                } else {
                    let deps: Vec<String> = sparks.sparks.iter().map(|(s, _)| s.clone()).collect();
                    let msg = format!(
                        "{} `{}`.\n{} [{}]",
                        loc!("Reactive block", "Реактивный блок"),
                        kind,
                        loc!("It's tracking these dependencies:", "Зависит от:"),
                        deps.join(", ")
                    );
                    self.logger.custom_log(
                        loc!("REACTIVE", "РЕАКТИВНОСТЬ"),
                        LoggerColor::Yellow,
                        &msg,
                    );
                }
            }

            FireworkAction::ReactiveBlockIgnore(_, sparks, _) => {
                let deps: Vec<String> = sparks.sparks.iter().map(|(s, _)| s.clone()).collect();
                let msg = format!(
                    "{}\n{}",
                    loc!(
                        "Merged reactive block (Issue #2 workaround).",
                        "Слияние реактивных блоков (Решение Issue #2)."
                    ),
                    loc!(
                        "This block contains UI but is nested inside another reactive block. To prevent glitchy partial updates, the compiler 'merged' them. The parent block will now track these dependencies too:",
                        "Этот условный блок вложен в другой, и внутри него есть UI. Чтобы интерфейс не глючил от частичных обновлений, компилятор применил 'слияние'. Он передал зависимости родительскому блоку:"
                    )
                );
                self.logger.custom_log(
                    loc!("MERGED", "СЛИЯНИЕ"),
                    LoggerColor::Yellow,
                    &format!("{} [{}]", msg, deps.join(", ")),
                );
            }

            FireworkAction::DynamicLoopBegin(_, _) => {
                let msg = loc!(
                    "Dynamic list starts here. The compiler enables the MicroRuntime for efficient diffing. Ensure your widgets inside have a unique `key:` prop.",
                    "Начало динамического списка. Компилятор включает микро-рантайм для эффективной перерисовки элементов. Обязательно добавьте пропс `key:` виджетам внутри."
                );
                self.logger
                    .custom_log(loc!("DYN LIST", "ДИН. СПИСОК"), LoggerColor::Blue, msg);
            }

            FireworkAction::SparkRef {
                name, root, is_mut, ..
            } => {
                let mode = if *is_mut { "&mut" } else { "&" };
                let msg = format!(
                    "{} `{}` -> `{}` ({}).",
                    loc!(
                        "Grabbing a reference to shared state:",
                        "Берем ссылку на глобальный стейт (shared):"
                    ),
                    root,
                    name,
                    mode
                );
                self.logger
                    .custom_log(loc!("SHARED", "ОБЩИЕ ДАННЫЕ"), LoggerColor::Cyan, &msg);
            }

            FireworkAction::DropSpark { name, .. } => {
                let msg = format!(
                    "{} `{}`. {}",
                    loc!("Dropping spark", "Конец жизни спарка"),
                    name,
                    loc!(
                        "The variable goes out of scope. The compiler safely returns its ownership back to static memory.",
                        "Переменная выходит из области видимости. Компилятор аккуратно возвращает владение данными обратно в статическую память."
                    )
                );
                self.logger
                    .custom_log(loc!("LIFETIME", "ПАМЯТЬ"), LoggerColor::White, &msg);
            }

            _ => {}
        }

        println!();
    }
}
