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
            if let Some(variable) = self.scope.variables.get(&root_name) {
                if variable.is_spark {
                    if !variable.is_mut {
                        self.errors.push(compile_error_spanned(
                            &i,
                            SPARK_MUT_REQUIRED_ERROR,
                        ));
                    }
                    
                    self.statement.action = FireworkAction::UpdateSpark(root_name);
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
            BinOp::ShrAssign(_)  => true,

            _ => false,
        };

        if is_mutation {
            if let Some(root_name) = get_root_variable_name(&i.left) {
                if let Some(variable) = self.scope.variables.get(&root_name) {
                    if variable.is_spark {
                        if !variable.is_mut {
                            self.errors.push(compile_error_spanned(
                                &i,
                                SPARK_MUT_REQUIRED_ERROR,
                            ));
                        }
                        
                        self.statement.action = FireworkAction::UpdateSpark(root_name);
                    }
                }
            }
        }

        visit::visit_expr_binary(self, i);
    }

    pub(crate) fn analyze_expr_method_call(&mut self, i: &'ast syn::ExprMethodCall) {
        if let Some(root_name) = get_root_variable_name(&i.receiver) {
            if let Some(variable) = self.scope.variables.get(&root_name) {
                if variable.is_spark {
                    let method_name = i.method.to_string();
                    
                    if !variable.is_mut {
                        self.errors.push(compile_error_spanned(
                            &i,
                            SPARK_MUT_REQUIRED_ERROR,
                        ));
                    }

                    // Только мутабельные методы, узнать это можно по типу спарка
                    // через хелпер, если это кастомный тип то используется хак и
                    // все методы считаются мутабельными
                    if is_mutable_method(&variable.variable_type, &method_name) {
                        self.statement.action = FireworkAction::UpdateSpark(root_name);
                    }
                }
            }
        }
    }
}
