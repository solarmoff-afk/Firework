// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub use super::super::*;

use crate::compiler::analyze::utils::check_expr::ExprAnalyzeResult;

impl Analyzer {
    /// Функция хэлпер для регистрации реактивного блока в IR. Реактивный блок это блок
    /// (условие, цикл, match) которые содержит реактивную переменную (спарк) в своём
    /// условии. Он забирает всё содержимое тело поэтому реактивный блок в реактином блоке
    /// не считается отдельным реактивным блоком. Также он вызывает visit метод через
    /// замыкание (visit_fn). Добавляет закрывающий блок (}) в конце блока, замыкание
    /// должно вернуть DelimSpan (спаны открывающей и закрывающей скобки)
    pub(crate) fn handle_reactive_block(
        &mut self,
        expr: ExprAnalyzeResult,
        is_loop: bool,
        open_code: String,
        action: FireworkAction,
        visit_fn: impl FnOnce(&mut Self) -> DelimSpan,
    ) {
        let sparks = expr.sparks;

        // Добавление к счётчику глубины. Это используется для форматирования вывода чтобы
        // определить сколько табов нужно
        self.lifetime_manager.scope.depth += 1;

        // Текущее состояние
        let state = self.reactive_block;
        let is_loop_state = self.is_loop;
        let first_block_state = self.context.first_ui_reactive_block.clone();

        // Стейтемент для открытия реактивного блока чтобы кодогенератор мог правильно
        // сгенерировать реактивный блок
        let mut open_statement = self.context.statement.clone();
        open_statement.string = open_code;

        // Нулевой эффект это эффект который не содержит спарков в условии. Он нужен чтобы
        // создать код который выполняется только при билде и навигации, а Event или
        // Reactive флэши его не трогают
        let mut is_null_effect = false;
        let condition_has_spark = !sparks.is_empty();

        // Если это эффект
        if let FireworkAction::ReactiveBlock(FireworkReactiveBlock::Effect, vec, _) = &action {
            // Нулевой эффект должен быть пустым
            is_null_effect = vec.is_empty();
        }

        // Вызов в любом случае, даже если в условии нет спарков
        self.reactive_block = Some((self.statement_index, is_loop));

        // Если в условии есть спарки то мы входим в реактивный блок. Реактивные блоки
        // в реактивных блоках не работают. То есть реактивный блок будет создан если в
        // условии есть спарки или если это эффект без спарков. Если это эффект у которого
        // есть спарки то это сделает true condition_has_spark, а если это эффект без спарков
        // то is_null_effect
        let is_reactive = condition_has_spark || is_null_effect;

        if is_reactive {
            open_statement.action = action.clone();
            open_statement.is_reactive_block = true;
        } else {
            // Иначе это может быть else реактивного блока
            open_statement.action = FireworkAction::ReactiveElse;

            // Если это не else то обычный код
            if !matches!(action, FireworkAction::ReactiveElse) {
                open_statement.action = FireworkAction::DefaultCode;
            }

            open_statement.is_reactive_block = false;
        }

        // Открывающий стейтемент реактивного блока
        self.context.ir.push(open_statement);

        if !self.is_loop {
            self.is_loop = is_loop;
        }

        // Если это реактивный блок то он добавляется в стэк реактивных блоков. Необходимо
        // испоьзовать только в условии на is_reactive чтобы не было паники
        let mut hook: Option<IrHook> = None;

        if is_reactive && let Some(last_hook) = self.get_hook() {
            hook = Some(last_hook.clone());
            self.context.reactive_block_stack.push(last_hook.clone());

            // Если хук на первый реактивный блок не установлен то он устанавливается, этот
            // блок находится здесь так как это нужно выполнить до выполнение замыкания и
            // обхода остального дерева
            if self.context.first_ui_reactive_block.is_none() {
                self.context.first_ui_reactive_block = Some(last_hook);
            }
        }

        self.statement_index += 1;

        // let _saved_action = self.statement.action.clone();
        self.context.statement.action = FireworkAction::DefaultCode;

        // До входа в блок все спарки условия добавляются в стэка и создаётся копия
        // стэка чтобы после выхода из блока все спарки из этого условия исчезли из
        // стэка
        let spark_stack_snapshot = self.context.spark_stack.clone();
        self.context.spark_stack.extend(sparks);
        self.context.spark_stack.dedup();

        // Замыкание чтобы выполнить все блоки, self передаётся из-за того что в
        // расте нельзя использовать self внутри метода этой же структуры поэтому
        // здесь передаётся self как аргумент замыкания
        let delim_span = visit_fn(self);

        // После выполнения обработки блока идёт снятие реактивного блока из стэка
        if is_reactive && let Some(raw_hook) = hook {
            self.context.reactive_block_stack.pop();

            // Хук задаётся только если условие is_reactive верно, а здесь это условие
            // есть выше. Это нужно так как action из аргументов уже не является актуальным,
            // так как is_ui заполняет visit_fn
            let mut statement = self.get_statement_from_hook(raw_hook.clone()).clone();

            // После replace значение будет возвращено ниже
            let action = std::mem::replace(&mut statement.action, FireworkAction::DefaultCode);

            // Если это не эффект до этого не было реактивных блоков то хук помещается в
            // контекст как первый ui реактивный блок. Это нужно для того чтобы выполнить
            // слияние условий
            let new_action = match action {
                FireworkAction::ReactiveBlock(block_type, sparks, is_ui)
                    if block_type != FireworkReactiveBlock::Effect =>
                {
                    if let Some(ref first_hook) = first_block_state {
                        self.merge_guard(first_hook.clone(), &sparks);
                        FireworkAction::ReactiveBlockIgnore(block_type, sparks, is_ui)
                    } else {
                        FireworkAction::ReactiveBlock(block_type, sparks, is_ui)
                    }
                }

                action => action,
            };

            statement.action = new_action;

            // Чтобы обойти бороу чекер работа идёт с клоном, а потом оригинал заменяется на
            // клон по мутабельной ссылке
            let statement_ref = self.get_statement_from_hook(raw_hook.clone());
            *statement_ref = statement;
        }

        // Восстановление стэка спарков
        self.context.spark_stack = spark_stack_snapshot;

        // Закрывающий стейтемент реактивного блока
        self.context.statement.action = FireworkAction::ReactiveBlockTerminator;
        self.context.statement.string = "}".to_string();
        self.context.statement.span = delim_span.close();
        self.context.ir.set_span(delim_span.close());
        self.statement_index += 1;

        // Закрывающая фигурная скобка также является частью реактивного блока
        self.context.statement.is_reactive_block = true;

        self.reactive_block = state;
        self.is_loop = is_loop_state;
        self.context.first_ui_reactive_block = first_block_state;

        // Защита от переполнения
        if self.lifetime_manager.scope.depth > 0 {
            self.lifetime_manager.scope.depth -= 1;
        }
    }

    /// Этот метод принимает хук на нужный реактивный блок, а также спарки от другого
    /// реактивного блока. Если в хуке эффект то он добавляет все эти спарки к спаркам
    /// реактивного блока на который указывает хук, а также удаляет дубликаты
    #[doc = "Firework/issues/2"]
    fn merge_guard(&mut self, hook: IrHook, child_sparks: &ExprAnalyzeResult) {
        let statement = self.get_statement_from_hook(hook);

        if let FireworkAction::ReactiveBlock(_type, sparks, _is_ui) = &mut statement.action {
            sparks.extend(child_sparks);
            sparks.dedup();
        }
    }
}
