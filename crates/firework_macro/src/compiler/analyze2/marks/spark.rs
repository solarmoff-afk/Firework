// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub use super::super::*;

use super::super::type_inference::guess_type_from_expr;

impl Analyzer {
    /// Маркер spark!()
    pub(crate) fn spark_marker<'ast>(&mut self, i: &'ast Local) {
        // Найден ли спарк в правой части
        let mut found_spark = false;
        
        if let Some(local_init) = &i.init {
            // Валидатор сам сделает подсчёт спарков в выражении
            let mut validator = SparkValidator {
                spark_count: 0,
                spark_tokens: None,
                spark_expr: None,
            };
            
            // Вызов из валидатора
            validator.visit_expr(&local_init.expr);
            
            if validator.spark_count > 1 {
                // FE006 нельзя делать выражения с несколькими инициализациями спарков
                self.context.errors.push(compile_error_spanned(
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
                let mut _spark_content = "".to_string();
        
                for (name, mut var_data) in self.pending_vars.drain(..) {
                    var_data.is_spark = found_spark;
                    
                    if found_spark {
                        // SAFETY: Unwrap не вызовет паники так как мы находимся в блоке
                        // found_spark, а found_spark это истина только когда количество
                        // спарков в выражении это 1 (не 0 и не 2), валидатор добавляет
                        // единицу к этомк полю только когда находит spark! и в том же
                        // блоке заполняет spark_tokens как Some, а если он Some то паники
                        // быть не может при использовании unwrap
                        _spark_content = validator.spark_tokens.as_ref().unwrap().to_string();
                        
                        self.context.spark_counter += 1; 
                        
                        // FE002, нельзя затенять существующую переменную спарком
                        if self.lifetime_manager.scope.variables.contains_key(&name) {
                        self.context.errors.push(compile_error_spanned(
                            &i.pat,
                            SPARK_SHADOWING_ERROR,
                        ));
                    }
                    
                    let mut spark_type = var_data.variable_type.clone();
                    if var_data.variable_type == NO_TYPE.to_string() {
                        let mut guessed_type = None;
                        
                        // Если содержимое маркера удалось распарсить как выражение то
                        // нужно запустить тайп чекер для базовой проверки типа
                        if let Some(expr) = &validator.spark_expr {
                            guessed_type = guess_type_from_expr(expr);

                            // Если сработала ветка variable_type == NO_TYPE и мы не можем
                            // угадать тип то этап анализации завершится с ошибкой и мы
                            // не попадём в кодогенерацию (второй этап) поэтому можно
                            // задать любое значение в качестве типа, даже пустое
                            spark_type = guessed_type.clone().unwrap_or("".to_string());
                        }

                        // Если получилось угадать тип
                        if let Some(ty) = guessed_type {
                            var_data.variable_type = ty; 
                        } else {
                            // FE003, у спарка должен быть тип данных, например u32:
                            // let mut spark1: u32 = spark!(10); 
                            self.context.errors.push(compile_error_spanned(
                                &i.pat,
                                SPARK_TYPE_ERROR,
                            ));
                        }
                    }

                    temp_fields_to_struct.push((
                        format!("spark_{}", self.context.spark_counter),
                        spark_type,
                    ));
                    
                    var_data.spark_id = self.context.spark_counter;
                    self.context.statement.action = FireworkAction::InitialSpark {
                        name: name.clone(),
                        id: self.context.spark_counter,
                        spark_type: var_data.clone().variable_type,
                        expr_body: _spark_content,
                        is_mut: var_data.is_mut,
                    };
                }
                
                // FE004, нельзя затенить спарк
                if let Some(value) = self.lifetime_manager.scope.variables.get(&name) {
                    if value.is_spark { 
                        self.context.errors.push(compile_error_spanned(
                            &i.pat,
                            SPARK_UNIQUE_NAME_ERROR,
                        ));
                    }
                }
                
                self.lifetime_manager.scope.variables.insert(name, var_data);
            }
            
            for (field_name, field_type) in temp_fields_to_struct.iter() {
                self.add_field_to_struct(field_name.to_string(), field_type.to_string());
            }
        }
    }
}
