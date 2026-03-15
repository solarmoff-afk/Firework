// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use syn::*;
use syn::visit::Visit;
use quote::ToTokens;

use super::Scope;

/// Валидатор реактивных инициализаций (спарков)
pub struct SparkValidator {
    // Сколько вызовов spark! найдено в текущем выражении (expr). Это важно чтобы
    // избежать выражений spark!() + spark!() которые нельзя нормально разместить
    // в статическом графе зависимостей
    pub spark_count: usize,
}

impl<'ast> Visit<'ast> for SparkValidator {
    fn visit_expr_macro(&mut self, i: &'ast ExprMacro) {
        // Проверка что вызов действительно spark!
        if i.mac.path.is_ident("spark") {
            // Добавление единицы к значению всех вызовов spark в выражении 
            self.spark_count += 1;
        }

        // Возможно макрос внутри макроса, тут запуск парсинга такого выражения
        // работает потому-что даже если внутри макроса spark!(...) будет макрос
        // spark!(...) то добавится self.spark_count и он уже не будет равен 0 
        // или 1, а значит будет ошибка
        visit::visit_expr_macro(self, i);
    }
}

pub struct SparkFinder<'a> {
    pub scope: &'a Scope,
    pub found: &'a mut Vec<String>,
}

impl<'ast> Visit<'ast> for SparkFinder<'_> {
    fn visit_expr_path(&mut self, i: &'ast ExprPath) {
        let var_name = i.path.to_token_stream().to_string();

        if let Some(var) = self.scope.variables.get(&var_name) {
            if var.is_spark {
                if !self.found.contains(&var_name) {
                    self.found.push(var_name);
                }
            }
        }
    }
}

/// Эта функция позволяет узнать корень выражения чтобы потом понять явлется ли это
/// работой со спарком или нет в выражениях с полями на более высоком уровне анализатора
/// Пример:
///  spark1.field.subfield = 5;
///
///   spark1 - Корень
///   field1 - Поле
///   subfield - Поле
///
/// Зная это выражение нам нужно получить имя корня и вернуть его
pub fn get_root_variable_name(expr: &Expr) -> Option<String> {
    match expr {
        // Прямое использование спарка
        Expr::Path(path_expr) => {
            Some(path_expr.to_token_stream().to_string())
        },

        // Через поле, например spark1.field. В таком случае нужно запустить эту
        // функцию для базы (левого соседа) выражения и если там будет например
        // spark1.field.subfield то рекурсия выполнится для базы subfield,
        // дальше функция найдёт field и поймёт что это тоже поле и на 2 вызов
        // рекурсии найдёт корень (path), в этом примере это spark1
        Expr::Field(field_expr) => {
            get_root_variable_name(&field_expr.base)
        },

        // Индексация (например в векторах или массивах) spark1[10]
        Expr::Index(index_expr) => {
            get_root_variable_name(&index_expr.expr)
        },

        // Корень не найден
        _ => None
    }
}
