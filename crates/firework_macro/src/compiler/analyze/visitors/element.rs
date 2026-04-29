// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use syn::spanned::Spanned;

pub use super::super::*;

use crate::compiler::analyze::expr::widget::is_functional_widget;
use crate::compiler::codegen::ir::WidgetDescription;

impl<'ast> Analyzer {
    /// Макрос который используются не в выражении, а как отдельный statement (команда)
    pub(crate) fn analyze_macro(&mut self, i: &'ast Macro) {
        let name = i.path.to_token_stream().to_string();
        self.context.statement.span = i.path.span();

        if self.validate_element(i, &name) {
            return;
        }
       
        // Лайаут это конструкция layout_name! { // Обычный раст код }; все токены
        // внутри нужно распарсить как обычный раст код через block (Не file и не expr)
        // установка настроек лайаута идёт через специальный функциональный виджет
        // layout!(...); сам лайаут это лишь контейнер для виджетов. Такой подход
        // позволяет оставить DSL только на уровне виджетов, а весь остальной код
        // держать как чистый раст
        if is_layout(&name) {
            self.handle_layout(i);
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
                Err(_e) => {
                    // Ошибка FE007, нарушение синтаксиса DSL виджета. Синтаксис только
                    //
                    // widget_name! {
                    //   field1: 10,
                    //   field2: 20,
                    // }
                    self.context.errors.push(compile_error_spanned(
                        i.tokens.clone(),
                        WIDGET_PARSE_ERROR, 
                    ));

                    // Выход чтобы не продолжать
                    return;
                }
            };

            let mut fields_map: HashMap<String, FireworkWidgetField> = HashMap::new();

            let mut key_type = "u64".to_string();
            let mut has_key = false; 

            for prop in args.properties {
                let prop_name = prop.name.to_string();
                if prop_name == "key" {
                    has_key = true;
                }

                let mut this_field = FireworkWidgetField {
                    sparks: Vec::new(),
                    string: prop.value.to_token_stream().to_string(),
                    token_stream: prop.value.to_token_stream(),
                    
                    // Изначально это не замыкание
                    is_fn: false,
                };
                
                let mut finder = SparkFinderWithId {
                    scope: &self.lifetime_manager.scope,
                    found: &mut this_field.sparks,
                };

                finder.visit_expr(&prop.value);

                if let Expr::Closure(closure) = &prop.value { 
                    let saved_parent = self.context.statement.parent_widget_id;
                    
                    self.context.statement.parent_widget_id =
                        Some(self.context.widget_counter);
                    self.visit_expr(&closure.body);
                    self.context.statement.parent_widget_id = saved_parent;

                    // Если выражение это Closure то поле является замыканием
                    this_field.is_fn = true;
                }

                fields_map.insert(prop_name, this_field);

                if let Some(attr) = prop.get_attribute("key_type") {
                    if let Some(args) = &attr.args {
                        if let Some(first) = args.first() {
                            key_type = first.to_string();
                        }
                    }
                }
            }

            if !has_key {
                self.context.errors.push(compile_error_spanned(
                    i.tokens.clone(),
                    WIDGET_KEY_REQUIRED_ERROR,
                ));
            }

            // Если в инициализации виджета есть поле skin то это должна быть структура с
            // методом build
            let mut _skin_struct: Option<String> = None;
            
            if let Some(skin) = fields_map.get("skin") {
                _skin_struct = Some(skin.string.clone());
            } else {
                // Иначе нужно использовать метод чтобы получить стандартную структуру
                // для скина этого виджета
                _skin_struct = map_skin(&name);
            }

            // Микрорантайм это контейнер который может хранится на куче (Vec или Smallvec)
            // и нужен в динамических списках чтобы разместить хэндлы рендер движка для
            // примитивных объектов созданных из виджетов внутри цикла 
            let mut has_microruntime = false;
            
            // У функциональных виджетов нет скина, а если поле _skin_struct пустое
            // то значит это функиональный виджет
            let mut skin_field = _skin_struct.clone().unwrap_or("".to_string());
            
            if let Some((_start, _need_microruntime)) = self.reactive_block {
                has_microruntime = _need_microruntime;

                if _need_microruntime {
                    // Если виджет был декларирован в цикле то его нужно обернуть в
                    // специальный контейнер
 
                    self.context.microruntime_widgets.has_widgets = true; 

                    skin_field = format!("firework_ui::DynList<{}, {}>", key_type, skin_field);
                    self.context.ir.screen_dynamic_widgets
                        .entry(self.context.statement.screen_index)
                        .or_insert_with(Vec::new)
                        .push(self.context.widget_counter);
                }
            }

            // Только если в skin_struct была добавлена структура нужно добавить поле
            // в структуру экрана. Если поля нет то это функциональный виджет который
            // не получил скин через поле skin
            self.add_field_to_struct(
                format!("widget_object_{}", self.context.widget_counter),
                skin_field.to_string(),
            );

            self.context.statement.string = i.to_token_stream().to_string();
            self.context.statement.action = FireworkAction::WidgetBlock(
                WidgetDescription {
                    widget_type: name.clone(),
                    fields: fields_map,
                    is_functional: is_functional_widget(&name),
                    id: self.context.widget_counter,
                    has_microruntime, 
                    skin: _skin_struct.unwrap_or("".to_string()),

                    is_maybe: if self.context.is_maybe {
                        for spark in &self.context.spark_stack {
                            self.context.spark_widget_map.entry(spark.1)
                                .or_insert_with(Vec::new)
                                .push(self.context.maybe_widgets_counter);
                        }

                        Some(self.context.maybe_widgets_counter)
                    } else {
                        None
                    },
                }
            );
            self.context.ir.push(self.context.statement.clone());
            self.statement_index += 1;

            if has_microruntime {
                self.context.microruntime_widgets.widgets.push(self.context.widget_counter);
                self.context.microruntime_widgets.count += 1;
            }

            // Если внутри реактивного блока происходит декларация UI то стандартное условие
            // выполнить если фаза Build или navigate или состояние изменилось не работает,
            // так как при клике (Event) все три условия не выполняются. Это критично для
            // динамических списков так как они требуют чтобы кто-то их дёргал. Если внутри
            // реактивного блока есть декларация виджета (не важно обычного, условного или
            // динамического) то он и все родительские реактивные блоки в стэке помечаются
            // как часть декларации UI и в условии также будет выполнение при Event, а обычные
            // реактивные блоки без виджетов внутри не будут затронуты
            let stack = self.context.reactive_block_stack.clone();
            for reactive_hock in &stack {
                let statement = self.get_statement_from_hook(reactive_hock.clone());

                if let FireworkAction::ReactiveBlock(_, _, is_ui) = &mut statement.action {
                    *is_ui = true;
                }
            }

            self.context.widget_counter += 1;

            // Если вызов находится в условии или паттерн матчинге (Match) то виджет является
            // условным
            if self.context.is_maybe {
                self.context.maybe_widgets_counter += 1;
            }
        } else if name == "effect" {
            self.effect_marker(i);
        }

        visit::visit_macro(self, i);
    }

    fn handle_layout(&mut self, i: &'ast Macro) {
        let name = i.path.to_token_stream().to_string();
        let inner_tokens = &i.tokens;

        // Микрорантайм это специальный тип данных firework_ui::DynList
        let mut has_microruntime = false;
        if let Some((_start, need_microruntime)) = self.reactive_block {
            has_microruntime = need_microruntime;
        }

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

            self.context.statement.action = FireworkAction::LayoutBlock(
                name.clone(), has_microruntime,
            );
            
            self.context.statement.screen_index = self.lifetime_manager.scope.screen_index;
            self.context.statement.depth = self.lifetime_manager.scope.depth;
            self.context.ir.push(self.context.statement.clone());

            self.lifetime_manager.scope.depth += 1;
            self.context.statement.depth += 1;

            // Добавление перед парсингом вложенных команд
            self.context.layouts_count += 1;

            for statement in block.stmts { 
                // Парсинг всех команд внутри
                self.visit_stmt(&statement);
            }

            self.context.layouts_count -= 1; 
           
            self.lifetime_manager.scope.depth -= 1;
            self.context.statement.depth -= 1;
            self.context.statement.action = FireworkAction::DefaultCode;
            self.context.statement.string = "}".to_string();
            self.statement_index += 1;

            self.context.ir.push(self.context.statement.clone()); 
        } else {
            // FE008, невалидный синтаксис в лайауте. Как уже было сказанно ранее,
            // лайаут требует полностью валидный раст синтаксис
            self.context.errors.push(compile_error_spanned(
                i.tokens.clone(),
                LAYOUT_PARSE_ERROR,
            ));

            return;
        }
    }

    fn validate_element(&mut self, i: &'ast Macro, name: &str) -> bool {
        // Проверка что лайаут конфигурируется только один раз в лайаут блоке
        if name == "layout" {
            if self.descript_layout {
                // FE009
                self.context.errors.push(compile_error_spanned(
                    // Весь макрос
                    i,
                    LAYOUT_MULTIPLE_ERROR,
                ));

                // Если это функциональный виджет layout то нужно выйти 
                return true;
            }

            self.descript_layout = true;
        }

        if (is_layout(&name) || is_widget(&name)) && !matches!(i.delimiter, MacroDelimiter::Brace(_)) {
            self.context.errors.push(compile_error_spanned(
                i,
                MACRO_BRACE_ERROR,
            ));
            
            return true;
        }

        false
    }
}
