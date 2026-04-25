// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use syn::punctuated::Punctuated;
use syn::parse::Parser;

pub use super::super::*;

use crate::CompileType;
use crate::compiler::codegen::ir::MaybeWidgets;

// Для того чтобы определить сколько нужео полей для снапшотов масок условных виджетов
use crate::compiler::codegen::generator::bitmask_gen::get_spark_mask;

impl<'ast> Analyzer {
    /// Генерирует заглушки для функций чтобы компилятор не выдал ошибку "функция отсуствует"
    /// вероятно это временное решение. Также собирает сигнатуру функции для кодогенератора
    pub(crate) fn analyze_item_fn(&mut self, node: &'ast ItemFn) {
        self.lifetime_manager.item_scope = self.lifetime_manager.scope.clone();
        self.context.layouts_count = 0;
        self.context.functions_count += 1;

        let mut function_head = String::from(""); 

        for attr in &node.attrs {
            function_head.push_str(format!("{}\n", quote::quote! { #attr }).as_str());

            let path = &attr.meta.path();
            let is_effect = path.is_ident("effect") || 
                (path.segments.len() == 2 && 
                path.segments[0].ident == "firework_ui" && 
                path.segments[1].ident == "effect");

            let is_shared = matches!(self.context.flags.compile_type, CompileType::Shared);
            if is_effect && matches!(&attr.meta, syn::Meta::List(_)) && is_shared {
                if let syn::Meta::List(list) = &attr.meta { 
                    let parser = Punctuated::<syn::Path, Token![,]>::parse_terminated;
                    
                    if let Ok(args) = parser.parse2(list.tokens.clone()) {
                        for arg in args {
                            if let Some(ident) = arg.get_ident() {
                                self.context.ir.shared.effects.entry(ident.to_string())
                                    .or_insert(Vec::new())
                                    .push(node.sig.ident.to_string());
                            }
                        }
                    }
                }
            }
        }
        
        let vis = &node.vis;
        let constness = &node.sig.constness;
        let asyncness = &node.sig.asyncness;
        let unsafety = &node.sig.unsafety;
        let abi = &node.sig.abi;
        let fn_token = &node.sig.fn_token;
        let ident = &node.sig.ident;
        let generics = &node.sig.generics;
        let inputs = &node.sig.inputs;
        let output = &node.sig.output;
        
        let signature = quote::quote! {
            #vis #constness #asyncness #unsafety #abi #fn_token #ident #generics (#inputs) #output
        };

        function_head.push_str(format!("{}", signature).as_str());

        // Добавление всех аргументов в область видимости как переменных
        for input in &node.sig.inputs {
            self.visit_fn_arg(input);
        }

        let function_name = node.sig.ident.to_string();
        self.function_name = Some(function_name.clone());
        self.context.ir.screens.push(
            (
                function_name.clone(),
                function_head,
                self.lifetime_manager.scope.screen_index
            )
        );
        self.context.statement.screen_name = function_name;

        // Любой код в функции реактивный
        self.context.statement.reactive_loop = true;

        self.generate_screen_id_field();

        // TODO: Нужно добавить проверку что если мы уже в функции экрана то другие функции
        // внутри не должны анализироваться и трансформироваться, там не ui контекст
        syn::visit::visit_item_fn(self, node);

        // Для Event событий в условном рендеринге нельзя обнулять маску, поэтому нужно
        // взять снапшот из статики на момент прошлого флэша и использовать его копию
        // вместо 0u64
        let mut widget_mask_count = get_spark_mask(self.context.maybe_widgets_counter);
        
        // Есило условных виджетов нет то генерировать поля для снапшотов масок не нужно,
        // так как get_spark_mask всегда возвращает значение >= 1, даже когда там ноль
        if self.context.maybe_widgets_counter == 0 {
            widget_mask_count = 0;
        }

        for mask_index in 0..widget_mask_count {
            self.add_field_to_struct(format!("_fwc_widget_bitmask{}", mask_index + 1),
                "u64".to_string());
        }

        // После парсинга функции нужно добавить стейтемент который уведомит
        // кодогенератор о завершении тела функции чтобы он перед этим сгенерировал
        // выход из цикла реактивности если этого ещё никто не сделал
        let mut statement = self.context.statement.clone();
        statement.action = FireworkAction::Terminator;

        // Больше не часть цикла реактивности
        statement.reactive_loop = false;

        self.context.ir.push(statement);
        self.context.ir.screen_sparks.insert(
            self.lifetime_manager.scope.screen_index, self.context.spark_counter);
        
        self.context.ir.screen_maybe_widgets.insert(self.lifetime_manager.scope.screen_index,
            MaybeWidgets {
                count: self.context.maybe_widgets_counter,
                spark_widget_map: self.context.spark_widget_map.clone(),
            }
        );

        // Очистка локальной карты так как экран уже обработан
        self.context.spark_widget_map.clear();

        // Сбрасывание нужно только если это не компиляция shared! {}, так как shared
        // это множество функций с общим состоянием, поэтому им нужна одна структура
        if !matches!(self.context.flags.compile_type, CompileType::Shared) {
            // Нужно сгенерировать индекс после анализации функции чтобы id экземпляра
            // был синхронизирован внутри блоков ir для одного экрана
            self.lifetime_manager.scope.screen_index_generate();
            
            // Обнуление счётчика реактивных переменных чтобы можно было считать что индекс
            // реактивной переменной это бит в битовой маске
            self.context.spark_counter = 0;
            self.linter.reset();
        }

        self.context.maybe_widgets_counter = 0;
    }

    pub(crate) fn analyze_fn_arg(&mut self, i: &'ast FnArg) {
        if let FnArg::Typed(pat_type) = i {
            self.current_type = pat_type.ty.to_token_stream().to_string();
        
            // Переменные будут добавляться в pending_vars
            self.visit_pat(&pat_type.pat);
        
            for (name, mut var_data) in self.pending_vars.drain(..) {
                // Аргумент функции не может быть спарком
                var_data.is_spark = false;

                self.lifetime_manager.scope.variables.insert(name, var_data);
            }

            self.current_type = String::from(NO_TYPE);
        }
    }

    /// Генерирует поле в структуре экрана/shared чтобы в ней всегда было одно поле
    /// даже если нет состояния
    fn generate_screen_id_field(&mut self) {
        // Если это компиляция shared юнита то у него должно быть только одно поле
        // _fwc_screen_id так как структура одна, это обеспечивается выходом из
        // генерации поля если количество функций которые были инициализированы
        // больше 1, то есть одна функция уже инициализировала это поле
        if matches!(self.context.flags.compile_type, CompileType::Shared)
                && self.context.functions_count > 1 {
            return;
        }

        // HACK: Быстрый фикс проблемы с тем, что если в экране не используются спарки
        // то структура не генерируется. Всегда добавляется _fwc_null на u8 (1 байт)
        self.add_field_to_struct("_fwc_screen_id".to_string(), "u8".to_string());
    }
}
