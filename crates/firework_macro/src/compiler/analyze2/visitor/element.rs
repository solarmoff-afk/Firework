// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub use super::super::*;

impl<'ast> Analyzer {
    /// Макрос который используются не в выражении, а как отдельный statement (команда)
    pub(crate) fn analyze_macro(&mut self, i: &'ast Macro) {
        let name = i.path.to_token_stream().to_string(); 

        // Проверка что лайаут конфигурируется только один раз в лайаут блоке
        if name == "layout" {
            if self.descript_layout {
                // FE009
                self.errors.push(compile_error_spanned(
                    // Весь макрос
                    i,

                    LAYOUT_MULTIPLE_ERROR,
                ));

                return;
            }

            self.descript_layout = true;
        }

        if (is_layout(&name) || is_widget(&name)) && !matches!(i.delimiter, MacroDelimiter::Brace(_)) {
            self.errors.push(compile_error_spanned(
                i,
                MACRO_BRACE_ERROR
            ));
            
            return;
        }
       
        // Лайаут это конструкция layout_name! { // Обычный раст код }; все токены
        // внутри нужно распарсить как обычный раст код через block (Не file и не expr)
        // установка настроек лайаута идёт через специальный функциональный виджет
        // layout!(...); сам лайаут это лишь контейнер для виджетов. Такой подход
        // позволяет оставить DSL только на уровне виджетов, а весь остальной код
        // держать как чистый раст
        if is_layout(&name) {
            let inner_tokens = &i.tokens;

            let parse_result: syn::Result<Block> = syn::parse2(quote::quote! {
                {
                    #inner_tokens
                }
            });

            if let Ok(block) = parse_result {
                // До входа в новый лайаут блок снимает флаг чтобы при конфигурации
                // нового лайаута не было FE009, делаем это до прохода по командам
                // внутри лайаут блока
                self.descript_layout = false;

                let mut has_microruntime = false;
                if let Some((start, need_microruntime)) = self.reactive_block {
                    has_microruntime = need_microruntime;
                }

                self.statement.action = FireworkAction::LayoutBlock(
                    name.clone(), has_microruntime,
                );
                
                self.statement.scope = self.scope.clone(); 
                self.ir.statements.push(self.statement.clone());

                self.scope.depth += 1;
                self.statement.scope.depth += 1;

                for statement in block.stmts { 
                    // Парсинг всех команд внутри
                    self.visit_stmt(&statement);
                }
               
                self.scope.depth -= 1;
                self.statement.scope.depth -= 1;
                self.statement.action = FireworkAction::DefaultCode;
                self.statement.string = "}".to_string();
                self.statement_index += 1;

                self.ir.statements.push(self.statement.clone());
            } else {
                // FE008, невалидный синтаксис в лайауте. Как уже было сказанно ранее,
                // лайаут требует полностью валидный раст синтаксис
                self.errors.push(compile_error_spanned(
                    i.tokens.clone(),
                    LAYOUT_PARSE_ERROR,
                ));

                return;
            }
        } else if is_widget(&name) {
            // Виджет это строительный блок ui, имеет синтаксис widget_name!(field: 123);
            // в отличии от лайаута там не валидный rust код, а специальный DSL который
            // похож на конструкцию структур. Пример:
            //
            // rect! {
            //  image: "test.png".to_string(),
            // }
            let args: WidgetArgs = match syn::parse2(i.tokens.clone()) {
                Ok(args) => args,
                Err(e) => {
                    // Ошибка FE007, нарушение синтаксиса DSL виджета. Синтаксис только
                    //
                    // widget_name@ {
                    //   field1: 10,
                    //   field2: 20,
                    // }
                    self.errors.push(compile_error_spanned(
                        i.tokens.clone(),
                        WIDGET_PARSE_ERROR, 
                    ));

                    // Выход чтобы не продолжать
                    return;
                }
            };

            let mut fields_map: HashMap<String, FireworkWidgetField> = HashMap::new();

            for prop in args.properties {
                let prop_name = prop.name.to_string();
                let mut this_field = FireworkWidgetField {
                    sparks: Vec::new(),
                    string: prop.value.to_token_stream().to_string(),
                };
                
                let mut finder = SparkFinder {
                    scope: &self.scope,
                    found: &mut this_field.sparks,
                };

                finder.visit_expr(&prop.value);

                if let Expr::Closure(closure) = &prop.value { 
                    let saved_parent = self.statement.parent_widget_id;
                    
                    self.statement.parent_widget_id = Some(self.widget_counter);
                    self.visit_expr(&closure.body);
                    
                    self.statement.parent_widget_id = saved_parent;
                }

                fields_map.insert(prop_name, this_field);
            }

            if let Some(skin) = map_skin(&name) {
                self.add_field_to_struct(
                    format!("widget_object_{}", self.widget_counter),
                    skin,
                );
            }

            // [REFACTORME]
            // Убрать дубляж кода
            let mut has_microruntime = false;
            if let Some((start, need_microruntime)) = self.reactive_block {
                has_microruntime = need_microruntime;
            }

            self.statement.string = i.to_token_stream().to_string();
            self.statement.action = FireworkAction::WidgetBlock(
                name.clone(),
                fields_map,
                has_microruntime,
                self.widget_counter,
            );
            self.ir.statements.push(self.statement.clone());
            self.statement_index += 1;

            self.widget_counter += 1;
        }

        visit::visit_macro(self, i);
    }
}
