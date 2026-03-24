// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub use super::super::*;

impl<'ast> Analyzer {
    pub fn analyze_local(&mut self, i: &'ast Local) {
        // Очистка данных из старого let
        self.pending_vars.clear();
        visit::visit_pat(self, &i.pat);

        // Найден ли спарк в правой части
        let mut found_spark = false;

        if let Some(local_init) = &i.init {
            // Валидатор сам сделает подсчёт спарков в выражении
            let mut validator = SparkValidator {
                spark_count: 0,
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
            for (name, mut var_data) in self.pending_vars.drain(..) {
                var_data.is_spark = found_spark;
 
                if found_spark {
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
                        name.clone(), 0, var_data.clone().variable_type,
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
                self.add_field_to_struct(field_name.to_string(), field_type.to_string())
            }
        }
    }
}
