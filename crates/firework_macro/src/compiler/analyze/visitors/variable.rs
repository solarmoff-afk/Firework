// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub use super::super::*;

impl<'ast> Analyzer {
    /// Анализация создания локальной переменной через let
    pub(crate) fn analyze_local(&mut self, i: &'ast Local) {
        // Очистка данных из старого let
        self.pending_vars.clear();
        visit::visit_pat(self, &i.pat);

        // Spark
        //  Синтаксис: spark!(value)
        //  Что делает: Создаёт реактивную переменную которую отслеживает анализатор
        self.spark_marker(i);

        // Spark Ref
        //  Синтаксис: spark_ref!(имя)
        //  Что делает: Создаёт ссылку на данные shared! {} блока по имени которое было
        //   указанно в сегменте state! {}
        self.spark_ref_marker(i);

        // Обработка правой части выражения
        // {let mut name} {=} {19}
        //     Левая       = Правая
        if let Some(local_init) = &i.init {
            visit::visit_expr(self, &local_init.expr);
        }
    }

    // Обновление переменной, например:
    //  var += 1;
    //  var = 1;
    //  var.field = 1;
    //  var.mut_method();

    /// Присваивание значения к переменной которая инициализирована как спарк считаетсч
    /// обновлением состояния и требует обновления UI
    pub(crate) fn analyze_expr_assign(&mut self, i: &'ast ExprAssign) {
        if let Some(root_name) = get_root_variable_name(&i.left) {
            let mut errors: Vec<Error> = Vec::new();
            self.add_update_spark(
                root_name,
                || {
                    errors.push(compile_error_spanned(i, SPARK_MUT_REQUIRED_ERROR));
                },
                &i.right,
            );

            self.context.errors.extend(errors);
        }

        visit::visit_expr_assign(self, i);
    }

    /// Кейс обновления состояния для бинарных операций, например spark += 1 или
    /// spark %= 2, также требует обновления ui
    pub(crate) fn analyze_expr_binary(&mut self, i: &'ast ExprBinary) {
        // Является ли бинарная операция мутацией
        let is_mutation = matches!(
            i.op,
            BinOp::AddAssign(_)
                | BinOp::SubAssign(_)
                | BinOp::MulAssign(_)
                | BinOp::DivAssign(_)
                | BinOp::RemAssign(_)
                | BinOp::BitAndAssign(_)
                | BinOp::BitOrAssign(_)
                | BinOp::BitXorAssign(_)
                | BinOp::ShlAssign(_)
                | BinOp::ShrAssign(_)
        );

        if is_mutation && let Some(root_name) = get_root_variable_name(&i.left) {
            let mut errors: Vec<Error> = Vec::new();
            self.add_update_spark(
                root_name,
                || {
                    errors.push(compile_error_spanned(i, SPARK_MUT_REQUIRED_ERROR));
                },
                &i.right,
            );

            self.context.errors.extend(errors);
        }

        visit::visit_expr_binary(self, i);
    }

    pub(crate) fn analyze_expr_method_call(&mut self, i: &'ast syn::ExprMethodCall) {
        if let Some(root_name) = get_root_variable_name(&i.receiver)
            && let Some(variable) = self.lifetime_manager.scope.variables.get(&root_name)
            && variable.is_spark
        {
            let method_name = i.method.to_string();

            if !variable.is_mut {
                self.context
                    .errors
                    .push(compile_error_spanned(i, SPARK_MUT_REQUIRED_ERROR));
            }

            // Только мутабельные методы, узнать это можно по типу спарка
            // через хелпер, если это кастомный тип то используется хак и
            // все методы считаются мутабельными
            if is_mutable_method(&variable.variable_type, &method_name) {
                self.context.statement.action = FireworkAction::UpdateSpark(
                    root_name,
                    variable.spark_id,
                    variable.is_spark_ref.clone(),
                );
                self.context.ir.push(self.context.statement.clone());
            }
        }
    }

    /// Добавляет UpdateSpark метку в текущий буферный виртуальныый стейтемент
    fn add_update_spark<F>(&mut self, root_name: String, mut mut_error: F, expr: &Expr)
    where
        F: FnMut(),
    {
        if let Some(variable) = self.lifetime_manager.scope.variables.get(&root_name)
            && variable.is_spark
        {
            if !variable.is_mut {
                mut_error();
            }

            self.context.statement.action = FireworkAction::UpdateSpark(
                root_name.clone(),
                variable.spark_id,
                variable.is_spark_ref.clone(),
            );

            // Клоинрование стейтемента перед передачей нужно для того чтобы
            // сохранилась семантическая метка (FireworkAction)
            self.compute_derived_spark(
                expr,
                self.context.statement.clone(),
                (&root_name, variable.spark_id),
            );
        }
    }
}
