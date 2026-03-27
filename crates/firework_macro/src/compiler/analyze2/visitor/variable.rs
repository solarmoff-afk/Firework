// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub use super::super::*;

impl<'ast> Analyzer {
    pub(crate) fn analyze_local(&mut self, i: &'ast Local) {
        // Очистка данных из старого let
        self.pending_vars.clear();
        visit::visit_pat(self, &i.pat);

        // Найден ли спарк в правой части
        let mut found_spark = false;

        if let Some(local_init) = &i.init {
            // Валидатор сам сделает подсчёт спарков в выражении
            let mut validator = SparkValidator {
                spark_count: 0,
                spark_tokens: None,
            };

            // Вызов из валидатора
            validator.visit_expr(&local_init.expr);

            if validator.spark_count > 1 {
                // FE006 нельзя делать выражения с несколькими инициализациями спарков
                self.errors.push(compile_error_spanned(
                    &local_init.expr,
                    SPARK_MULTIPLE_ERROR,
                ));
            }

            // SparkValidator нашёл один спарк в выражении
            if validator.spark_count == 1 {
                found_spark = true; 
            }

            // Временный вектор чтобы сложить туда поля, так как пушить нельзя из-за
            // мутабельной ссылки от drain
            let mut temp_fields_to_struct: Vec<(String, String)> = Vec::new();
            let mut spark_content = "".to_string();

            for (name, mut var_data) in self.pending_vars.drain(..) {
                var_data.is_spark = found_spark;
 
                if found_spark {
                    // SAFETY: Unwrap не вызовет паники так как мы находимся в блоке
                    // found_spark, а found_spark это истина только когда количество
                    // спарков в выражении это 1 (не 0 и не 2), валидатор добавляет
                    // единицу к этомк полю только когда находит spark! и в том же
                    // блоке заполняет spark_tokens как Some, а если он Some то паники
                    // быть не может при использовании unwrap
                    spark_content = validator.spark_tokens.as_ref().unwrap().to_string();
                    
                    temp_fields_to_struct.push((
                        format!("spark_{}", self.spark_counter),
                        var_data.clone().variable_type,
                    ));

                    self.spark_counter += 1;

                    // FE002, нельзя затенять существующую переменную спарком
                    if self.scope.variables.contains_key(&name) {
                        self.errors.push(compile_error_spanned(
                            &i.pat,
                            SPARK_SHADOWING_ERROR,
                        ));
                    }

                    // FE003, у спарка должен быть тип данных, например u32
                    // let mut spark1: u32 = spark!(10); 
                    if var_data.variable_type == NO_TYPE.to_string() {
                        self.errors.push(compile_error_spanned(
                            &i.pat,
                            SPARK_TYPE_ERROR,
                        ));
                    }

                    self.spark_counter += 1;
                    self.statement.action = FireworkAction::InitialSpark(
                        name.clone(), 0, var_data.clone().variable_type, spark_content,
                    );
                }

                // FE004, нельзя затенить спарк
                if let Some(value) = self.scope.variables.get(&name) {
                    if value.is_spark { 
                        self.errors.push(compile_error_spanned(
                            &i.pat,
                            SPARK_UNIQUE_NAME_ERROR,
                        ));
                    }
                }

                self.scope.variables.insert(name, var_data);
            }

            for (field_name, field_type) in temp_fields_to_struct.iter() {
                self.add_field_to_struct(field_name.to_string(), field_type.to_string());
            }
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
            if let Some(variable) = self.scope.variables.get(&root_name) {
                if variable.is_spark {
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
