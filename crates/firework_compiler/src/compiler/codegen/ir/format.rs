// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

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
                        loc!("--- Code Inspector ---", "--- Инспектор кода ---"),
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
                loc!("--- End of inspection ---", "--- Инспекция завершена ---"),
            );
            println!();
        }
    }

    fn explain_statement(&self, stmt: &FireworkStatement) {
        let code_line = stmt.string.trim().replace('\n', " ");

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
                    if *is_mut {
                        loc!(
                            "Detected reactive variable (Mutable).",
                            "Обнаружена реактивная переменная (Изменяемая)."
                        )
                    } else {
                        loc!(
                            "Detected reactive variable (Immutable).",
                            "Обнаружена реактивная переменная (Неизменяемая)."
                        )
                    },
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
                    loc!("Mutation detected on", "Обнаружена мутация переменной"),
                    name,
                    loc!(
                        "This will trigger an update of all UI elements that depend on it.",
                        "Это вызовет перерисовку всех элементов интерфейса, которые от нее зависят."
                    )
                );
                self.logger
                    .custom_log(loc!("UPDATE", "ОБНОВЛЕНИЕ"), LoggerColor::Green, &msg);
            }

            FireworkAction::WidgetBlock(desc) => {
                let msg = format!(
                    "{} `{}`.",
                    loc!("Widget detected:", "Обнаружен виджет:"),
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
                                    "Event handler. Will run on user interaction.",
                                    "Событие. Сработает при взаимодействии."
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
                                loc!("Reactive prop. Tied to:", "Реактивный пропс. Привязан к:"),
                                deps.join(", ")
                            );
                            self.logger
                                .custom_log("   ~", LoggerColor::Yellow, &reactive_msg);
                        } else {
                            let static_msg = format!(
                                "`{}`: {}",
                                prop_name,
                                loc!(
                                    "Static prop. Will not update dynamically.",
                                    "Статичный пропс. Не будет обновляться динамически."
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
                    FireworkReactiveBlock::Effect => "effect!",
                };

                if sparks.is_empty() {
                    let msg = format!(
                        "{} `{}`. {}",
                        loc!("Static condition:", "Статичное условие:"),
                        kind,
                        loc!(
                            "No reactive dependencies found. Will only run once.",
                            "Реактивные зависимости не найдены. Выполнится только один раз."
                        )
                    );
                    self.logger
                        .custom_log(loc!("BLOCK", "БЛОК"), LoggerColor::White, &msg);
                } else {
                    let deps: Vec<String> = sparks.sparks.iter().map(|(s, _)| s.clone()).collect();
                    let msg = format!(
                        "{} `{}`. {} [{}]",
                        loc!("Reactive condition:", "Реактивное условие:"),
                        kind,
                        loc!("Triggers on changes to:", "Перезапустится при изменении:"),
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
                    "{} {} [{}]",
                    loc!(
                        "Merged condition. UI detected inside a nested block.",
                        "Склеенное условие. Обнаружен вложенный UI."
                    ),
                    loc!(
                        "Dependencies moved to the parent block:",
                        "Зависимости переданы родительскому блоку:"
                    ),
                    deps.join(", ")
                );
                self.logger
                    .custom_log(loc!("MERGE", "СЛИЯНИЕ"), LoggerColor::Yellow, &msg);
            }

            FireworkAction::DynamicLoopBegin(_, _) => {
                let msg = loc!(
                    "Dynamic list detected. Widgets inside must have a unique `key` prop.",
                    "Обнаружен динамический список. Виджеты внутри должны иметь уникальный пропс `key`."
                );
                self.logger
                    .custom_log(loc!("LIST", "СПИСОК"), LoggerColor::Blue, msg);
            }

            FireworkAction::SparkRef { name, root, .. } => {
                let msg = format!(
                    "{} `{}` {} `{}`.",
                    loc!("Linked shared state", "Привязано общее состояние"),
                    root,
                    loc!("to local variable", "к локальной переменной"),
                    name
                );
                self.logger
                    .custom_log(loc!("SHARED", "ОБЩИЕ ДАННЫЕ"), LoggerColor::Cyan, &msg);
            }

            FireworkAction::DropSpark { name, .. } => {
                let msg = format!(
                    "{} `{}`. {}",
                    loc!("Variable", "Переменная"),
                    name,
                    loc!(
                        "left scope. Reactivity tracking ended.",
                        "вышла из области видимости. Отслеживание завершено."
                    )
                );
                self.logger
                    .custom_log(loc!("SCOPE", "ОБЛАСТЬ"), LoggerColor::White, &msg);
            }

            _ => {}
        }

        println!();
    }
}
