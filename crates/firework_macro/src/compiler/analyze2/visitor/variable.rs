// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub use super::super::*;

impl<'ast> Analyzer {
    /// Анализация создания локальной переменной через let
    pub(crate) fn analyze_local(&mut self, i: &'ast Local) {
        // Очистка данных из старого let
        self.pending_vars.clear();
        visit::visit_pat(self, &i.pat);

        self.spark_marker(i);
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
            if let Some(variable) = self.lifetime_manager.scope.variables.get(&root_name) {
                if variable.is_spark {
                    if !variable.is_mut {
                        self.context.errors.push(compile_error_spanned(
                            &i,
                            SPARK_MUT_REQUIRED_ERROR,
                        ));
                    } 

                    self.context.statement.action = FireworkAction::UpdateSpark(
                        root_name, variable.spark_id,
                    ); 

                    // Клоинрование стейтемента перед передачей нужно для того чтобы
                    // сохранилась семантическая метка (FireworkAction)
                    self.compute_spark(&i.right, self.context.statement.clone());
                }
            }
        }

        visit::visit_expr_assign(self, i);
    }

    /// Кейс обновления состояния для бинарных операций, например spark += 1 или
    /// spark %= 2, также требует обновления ui
    pub(crate) fn analyze_expr_binary(&mut self, i: &'ast ExprBinary) {
        // Является ли бинарная операция мутацией
        let is_mutation = match i.op {
            BinOp::AddAssign(_)   | BinOp::SubAssign(_)    | BinOp::MulAssign(_)    |
            BinOp::DivAssign(_)   | BinOp::RemAssign(_)    | BinOp::BitAndAssign(_) |
            BinOp::BitOrAssign(_) | BinOp::BitXorAssign(_) | BinOp::ShlAssign(_)    |
            BinOp::ShrAssign(_)   => true,

            _ => false,
        };

        if is_mutation {
            if let Some(root_name) = get_root_variable_name(&i.left) {
                if let Some(variable) = self.lifetime_manager.scope.variables.get(&root_name) {
                    if variable.is_spark {
                        if !variable.is_mut {
                            self.context.errors.push(compile_error_spanned(
                                &i,
                                SPARK_MUT_REQUIRED_ERROR,
                            ));
                        }
                        
                        self.context.statement.action = FireworkAction::UpdateSpark(
                            root_name, variable.spark_id,
                        );

                        self.compute_spark(&i.right, self.context.statement.clone());
                    }
                }
            }
        }

        visit::visit_expr_binary(self, i);
    }

    pub(crate) fn analyze_expr_method_call(&mut self, i: &'ast syn::ExprMethodCall) {
        if let Some(root_name) = get_root_variable_name(&i.receiver) {
            if let Some(variable) = self.lifetime_manager.scope.variables.get(&root_name) {
                if variable.is_spark {
                    let method_name = i.method.to_string();
                    
                    if !variable.is_mut {
                        self.context.errors.push(compile_error_spanned(
                            &i,
                            SPARK_MUT_REQUIRED_ERROR,
                        ));
                    }

                    // Только мутабельные методы, узнать это можно по типу спарка
                    // через хелпер, если это кастомный тип то используется хак и
                    // все методы считаются мутабельными
                    if is_mutable_method(&variable.variable_type, &method_name) {
                        self.context.statement.action = FireworkAction::UpdateSpark(
                            root_name, variable.spark_id,
                        );
                    }
                }
            }
        }
    }

    /// Этот метод реализует ищет спарки в правой части присваивания к реактивной переменной
    /// и если находит то оборачивает весь стейтемент в эффект который подписан на все
    /// спарки которые используются в варажении. Позволяет писать spark1 = spark2 + spark3
    /// без обёрток (как effect!(..., {})) и делать код интутивно понятным. Второй аргумент
    /// это стейтемент который будет вставлен в IR как внутрянка эффекта
    pub(crate) fn compute_spark(&mut self, right: &'ast Expr, mut statement: FireworkStatement) {
        let effect_sparks = self.get_sparks(&right);
        
        if effect_sparks.len() > 0 {
            self.handle_reactive_block(
                effect_sparks.clone(),
                false,
                "{ // effect".to_string(),
                FireworkAction::ReactiveBlock(FireworkReactiveBlock::Effect, effect_sparks),
                |this| {
                    // Так как условие if effect_sparks.len() > 0 { выше не сработало бы
                    // и этот код не выполнился бы если в выражении нет спарков то блок
                    // здесь точно реактивный. Это не затронет self.statement так как
                    // statement это клон
                    statement.is_reactive_block = true;

                    this.context.ir.push(statement);
                }
            ); 
        }
    }
}
