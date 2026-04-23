// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub use super::super::*;

use crate::CompileType;

impl Analyzer {
    /// Маркер spark_ref!(), используется в shared блоках чтобы взять мутабельную или
    /// немутабельную ссылку на поле в структуре shared блока (из сегмента state! {})
    pub(crate) fn spark_ref_marker<'ast>(&mut self, i: &'ast Local) { 
        if let Some(local_init) = &i.init { 
            let mut found_spark_ref = false;
            let mut ref_name = String::new();
            let mut is_valid = true;

            if self.find_spark_ref(&local_init.expr, &mut ref_name, &mut is_valid) {
                found_spark_ref = true;

                if !matches!(self.context.flags.compile_type, CompileType::Shared) {
                    self.context.errors.push(compile_error_spanned(
                        &local_init.expr,
                        SPARK_REF_CONTEXT_ERROR,
                    ));
                    
                    return;
                }
            }
            
            if !found_spark_ref {
                return;
            }
            
            if !is_valid {
                self.context.errors.push(compile_error_spanned(
                    &local_init.expr,
                    SPARK_REF_SYNTAX_ERROR,
                ));

                return;
            }
           
            // Проверка что выражение это только вызов макроса
            match *local_init.expr {
                Expr::Macro(_) => {},

                _ => {
                    self.context.errors.push(compile_error_spanned(
                        &local_init.expr,
                        SPARK_REF_COMPLEX_ERROR,
                    ));

                    return;
                }
            }

            for (name, mut var_data) in self.pending_vars.drain(..) {
                var_data.is_spark_ref = Some(name.clone());
                var_data.is_spark = true;

                if let Some(value) = self.lifetime_manager.scope.variables.get(&name) {
                    if value.is_spark {
                        self.context.errors.push(compile_error_spanned(
                            &i.pat,
                            SPARK_UNIQUE_NAME_ERROR,
                        ));
                    }
                }

                // Нужно чтобы в битовых масках был бит
                let id = self.context.spark_counter;
                self.context.spark_counter += 1;

                var_data.spark_id = id;
                self.linter.add_spark(id, name.clone(), i.to_token_stream().to_string());

                let mut found_name = false;
                for global_spark in &self.context.ir.shared.state { 
                    if global_spark.name == ref_name {
                        self.context.statement.action = FireworkAction::SparkRef {
                            name: name.clone(),
                            id: global_spark.id,
                            is_mut: var_data.is_mut,
                            root: ref_name.clone(),
                            local_id: id,
                        };

                        found_name = true;
                    }
                }

                if !found_name {
                    self.context.errors.push(compile_error_spanned(
                        &i.pat,
                        SPARK_REF_NOT_FOUND_ERROR,
                    ));
                }

                self.lifetime_manager.scope.variables.insert(name, var_data);
            }
        }
    }

    fn find_spark_ref(&self, expr: &Expr, ref_name: &mut String, is_valid: &mut bool) -> bool {
        match expr {
            Expr::Macro(macro_expr) => {
                if macro_expr.mac.path.is_ident("spark_ref") { 
                    let tokens = macro_expr.mac.tokens.clone();

                    match syn::parse2::<Expr>(tokens) {
                        Ok(Expr::Path(path_expr)) => {
                            *ref_name = path_expr.to_token_stream().to_string();
                            true
                        }
                        _ => {
                            *is_valid = false;
                            true
                        }
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}
