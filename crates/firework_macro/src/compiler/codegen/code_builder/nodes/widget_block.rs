// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use super::super::*;

impl CodeBuilder { 
    pub fn node_widget_block(
        &self, span: Span, struct_name: String, final_tokens: &mut TokenStream,
        statement: &FireworkStatement, 
    ) {
        match &statement.action {
            FireworkAction::WidgetBlock(
                widget_type, fields, _is_functional, id, has_microruntime, skin,
            ) => {
                let mut widget_init = "".to_string();

                // При навигации нужно сгенерировать конструкцию виджета на основе скина
                //                              [id лайаута]
                widget_init.push_str(format!("{}::new(1)", skin).as_str());

                // Обход всех полей
                for (name, field) in fields {
                    // Генерируется установка значения по билдер паттерну. Через точку
                    // вызывается метод, имя метода должено соотвестовать названию
                    // поля. Внутрь метода пробрасывается само значение
                    //
                    // rect! {
                    //  position: (10, 10),
                    // }
                    //
                    // Превращается в
                    // // Структура скина и айди лайаута в аргументах
                    // [SKIN]::new(1)
                    //  .position((10, 10)) // Имя поля становится вызовом метода
                    //                       // а вторая часть выражения становится
                    //                       // аргументов этого метода
                    widget_init.push_str(format!(".{}({})", name, field.string).as_str());
                }

                // Точка с запятой закрывает стейтемент
                widget_init.push(';');

                println!("Output: {}", widget_init);

                final_tokens.extend(quote_spanned!(span=> 
                    println!("Widget");
                ));
            },

            _ => {},
        };
    }
}
