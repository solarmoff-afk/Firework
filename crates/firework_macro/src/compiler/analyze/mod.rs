// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

mod components;
mod context;
mod expr;
mod lifetime;
mod linter;
mod marks;
mod type_inference;
pub mod utils;
mod visitors;

#[cfg(test)]
mod tests;

use proc_macro2::extra::DelimSpan;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use std::collections::HashMap;
use syn::visit::Visit;
use syn::*;

use context::AnalyzeContext;
use expr::spark::{SparkFinderWithId, SparkValidator, get_root_variable_name};
use expr::widget::{WidgetArgs, is_layout, is_widget, map_skin};
use lifetime::{LifetimeManager, Scope, Variable};
use linter::FireworkLinter;
use type_inference::mut_check::is_mutable_method;
use utils::hook::IrHook;

#[cfg(feature = "trace")]
use tracing::instrument;

#[cfg(feature = "trace")]
use quote::{quote, ToTokens};

use crate::CompileType;
use crate::compiler::CompileFlags;
use crate::compiler::codegen::ir::SpanKey;
use crate::compiler::codegen::ir::{
    FireworkAction, FireworkIR, FireworkReactiveBlock, FireworkStatement, FireworkWidgetField,
};
use crate::compiler::common::is_prop;
use crate::compiler::error::*;

/// Нельзя хранить String поэтому используется &str, при использовании нужно использовать
/// String::from, но это позволяет не тянуть lazy_static или другой крейт
pub const NO_TYPE: &str = "NO TYPE";

/// Главная структура анализатора для которого реализуется Visitor и который выполняет
/// роль анализа кода пользователя firework чтобы построить граф реактивности и
/// валидировать правильное использование спарков
pub struct Analyzer {
    pub context: AnalyzeContext,
    pub lifetime_manager: LifetimeManager,
    pub linter: FireworkLinter,

    // Statement это блок кода от начала до ; фигурных скобок или в некоторых случаях
    // запятой. Нужно точно знать на каком statement мы сейчас. На старте это 1, поэтому
    // итерацию нужно начать с единицы
    statement_index: usize,

    // Флаг который означает был ли использован функциональный виджет layout! в текущем
    // лайаут блоке. Описывать лайаут можно только один раз в лайаут блоке
    descript_layout: bool,

    // Текущее имя функции, если мы вне функции то None
    function_name: Option<String>,

    // Буферы
    // Буфер который используется для хранения текущего типа в парсинге переменной,
    // если типа не указан то используется None
    current_type: String,

    // Временный вектор имён переменных которые были найдены в текущем let, но ещё
    // не добавленных в scope.variables
    pending_vars: Vec<(String, Variable)>,

    // Маркер который показывает явлется ли этот statement частью реактивного блока.
    // Реактивный блок это условие, цикл (for/while) или match в условии которого
    // используется спарк. Если None то команда не в реактивном блоке, если Some(usize)
    // то строка в реактивном блоке, а usize это начало блока. Вложенные конструкции и
    // вложенные реактивные блоки не меняют этот флаг, он всегда показывает на первый
    // реактивный блок. Второе значение кортежа это цикл (нужен ли микрорантайм)
    reactive_block: Option<(usize, bool)>,

    // Нужен ли цикл
    is_loop: bool,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            context: AnalyzeContext::new(),
            lifetime_manager: LifetimeManager::new(),
            linter: FireworkLinter::new(),

            // Нулевая команда
            statement_index: 0,

            // Был ли настроен текущий лайаут, этот флаг нужен чтобы исключить двойной
            // вызов функционального виджета layout
            descript_layout: false,

            function_name: None,

            current_type: String::from(NO_TYPE),
            pending_vars: Vec::new(),

            // Изначально мы не в реактивном блоке
            reactive_block: None,

            is_loop: false,
        }
    }
}

impl<'ast> Visit<'ast> for Analyzer {
    /// Генерирует заглушки для функций чтобы компилятор не выдал ошибку "функция отсуствует"
    /// вероятно это временное решение. Также собирает сигнатуру функции для кодогенератора
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(node = %quote!(#i))))]
    fn visit_item_fn(&mut self, i: &'ast ItemFn) {
        self.analyze_item_fn(i);
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(node = %quote!(#i))))]
    fn visit_fn_arg(&mut self, i: &'ast FnArg) {
        self.analyze_fn_arg(i);
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(node = %quote!(#i))))]
    fn visit_local(&mut self, i: &'ast Local) {
        self.analyze_local(i);
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(node = %quote!(#i))))]
    fn visit_pat_type(&mut self, i: &'ast PatType) {
        // Это строка нужна чтобы запомнить текущий тип дпнных, это будет нужно в ветке
        // ident для определения типа, что потребуется в других ветках
        self.current_type = i.ty.to_token_stream().to_string();

        visit::visit_pat_type(self, i);

        // После завершения обработки нужно сбросить буфер
        self.current_type = String::from(NO_TYPE);
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(node = %quote!(#i))))]
    fn visit_pat_ident(&mut self, i: &'ast PatIdent) {
        self.pending_vars.push((
            i.ident.to_string(),
            Variable {
                variable_type: self.current_type.clone(),
                is_mut: i.mutability.is_some(),
                is_spark: false,
                spark_id: 0,
                is_spark_ref: None,
            },
        ));

        // На всякий случай
        visit::visit_pat_ident(self, i);
    }

    /// Вход в новую область видимости
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(node = %quote!(#i))))]
    fn visit_block(&mut self, i: &'ast syn::Block) {
        self.analyze_block(i);
    }

    /// Макрос который используются не в выражении, а как отдельный statement (команда)
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(node = %quote!(#i))))]
    fn visit_macro(&mut self, i: &'ast Macro) {
        self.analyze_macro(i);
    }

    /// Присваивание значения к переменной которая инициализирована как спарк считаетсч
    /// обновлением состояния и требует обновления UI
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(node = %quote!(#i))))]
    fn visit_expr_assign(&mut self, i: &'ast ExprAssign) {
        self.analyze_expr_assign(i);
    }

    /// Кейс обновления состояния для бинарных операций, например spark += 1 или
    /// spark %= 2, также требует обновления ui
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(node = %quote!(#i))))]
    fn visit_expr_binary(&mut self, i: &'ast ExprBinary) {
        self.analyze_expr_binary(i);
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(node = %quote!(#i))))]
    fn visit_expr_method_call(&mut self, i: &'ast syn::ExprMethodCall) {
        self.analyze_expr_method_call(i);
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(node = %quote!(#i))))]
    fn visit_stmt(&mut self, i: &'ast Stmt) {
        self.analyze_stmt(i);
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(node = %quote!(#i))))]
    fn visit_expr_if(&mut self, i: &'ast ExprIf) {
        self.analyze_expr_if(i);
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(node = %quote!(#i))))]
    fn visit_expr_while(&mut self, i: &'ast ExprWhile) {
        self.analyze_expr_while(i);
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(node = %quote!(#i))))]
    fn visit_expr_for_loop(&mut self, i: &'ast ExprForLoop) {
        self.analyze_expr_for_loop(i);
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(node = %quote!(#i))))]
    fn visit_expr_match(&mut self, i: &'ast ExprMatch) {
        self.analyze_expr_match(i);
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(node = %quote!(#i))))]
    fn visit_expr_loop(&mut self, i: &'ast ExprLoop) {
        self.analyze_expr_loop(i);
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(node = %quote!(#i))))]
    fn visit_expr_break(&mut self, i: &'ast ExprBreak) {
        self.analyze_expr_break(i);
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(node = %quote!(#i))))]
    fn visit_expr_continue(&mut self, i: &'ast ExprContinue) {
        self.analyze_expr_continue(i);
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(node = %quote!(#i))))]
    fn visit_expr_return(&mut self, i: &'ast ExprReturn) {
        self.analyze_expr_return(i);
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(node = %quote!(#i))))]
    fn visit_expr_closure(&mut self, i: &'ast ExprClosure) {
        self.analyze_expr_closure(i)
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(node = %quote!(#_i))))]
    fn visit_item_struct(&mut self, _i: &'ast ItemStruct) {
        if !matches!(self.context.flags.compile_type, CompileType::Component) {
            return;
        }

        let struct_name = _i.ident.to_string();

        match &_i.fields {
            Fields::Named(fields_named) => {
                for field in fields_named.named.iter() {
                    if let Some(ident) = &field.ident {
                        let field_name = ident.to_string();
                        let field_type_raw = &field.ty;
                        let field_type = quote!(#field_type_raw).to_string();

                        // Проверка на то, что поле структуры обёрнуто в Prop<T>
                        if !is_prop(&field_type) {
                            continue;
                        }

                        let props_vec = self.context
                            .component_props
                            .entry(struct_name.clone())
                            .or_default();
                        let len = props_vec.len();

                        // В качестве айди каждого пропса используется размер вектора пропсов
                        // компонента до добавления нового пропса. Это позволяет без нового
                        // счётчика генерировать айди для пропсов которое можно использовать
                        // для битов в битовой маске
                        props_vec.push((field_name, field_type, len));
                    }
                }
            }

            Fields::Unnamed(_) => {
                // TODO: Сделать ошибку
            }

            Fields::Unit => {}
        }
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(node = %quote!(#_i))))]
    fn visit_item_impl(&mut self, _i: &'ast ItemImpl) {
        if !matches!(self.context.flags.compile_type, CompileType::Component) {
            return;
        }

        let self_type = &_i.self_ty;

        if let Type::Path(type_path) = &**self_type {
            if let Some(segment) = type_path.path.segments.last() {
                let struct_name = &segment.ident;

                // Структура для которой идёт реализация становится текущим компонентом
                self.context.now_component = Some(struct_name.to_string())
            } else {
                return;
            }
        }

        for item in &_i.items {
            if let ImplItem::Fn(method) = item
                && method.sig.ident == "flash"
            {
                self.validate_flash_signature(method);
                self.analyze_item_fn(method);
            }
        }

        // Теперь никакой компонент не реализуется
        self.context.now_component = None;
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(node = %quote!(#i))))]
    fn visit_item_macro(&mut self, i: &'ast ItemMacro) {
        self.analyze_item_macro(i);
    }
}

pub fn prepare_tokens(
    file: File,
    flags: CompileFlags,
    _id: u64,
) -> (
    proc_macro2::TokenStream,
    Option<proc_macro2::TokenStream>,
    Option<FireworkIR>,
) {
    let mut analyzer = Analyzer::new();
    analyzer.lifetime_manager.scope.screen_index_generate();
    analyzer.context.flags = flags;
    analyzer.visit_file(&file);

    for warn_tokens in &analyzer.linter.warnings {
        analyzer.context.output.extend(warn_tokens.clone());
    }

    #[cfg(feature = "debug_output")]
    println!(
        "IR len: {}, IR: {:#?}",
        analyzer.context.ir.snapshot.statements.len(),
        analyzer.context.ir
    );

    if !analyzer.context.errors.is_empty() {
        let mut final_error = analyzer.context.errors[0].clone();

        for error in analyzer.context.errors.iter().skip(1) {
            final_error.combine(error.clone());
        }

        (
            analyzer.context.output,
            Some(final_error.to_compile_error()),
            Some(analyzer.context.ir),
        )
    } else {
        (analyzer.context.output, None, Some(analyzer.context.ir))
    }
}
