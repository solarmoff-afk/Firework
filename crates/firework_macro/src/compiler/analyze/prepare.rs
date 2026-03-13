// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use proc_macro2::TokenTree;
use syn::Error;
use std::collections::HashSet;

use crate::compiler::analyze::item::parse_items;
use crate::compiler::codegen::actions::{FireworkStatement, FireworkAction};

/// Структура которая собирает метаинформацию о каждой функции в ui блоке
#[derive(Clone)]
pub struct ItemMetadata {
    pub sparks: HashSet<String>,
    pub variables: HashSet<String>,
}

impl ItemMetadata {
    pub fn default() -> Self {
        Self {
            sparks: HashSet::new(),
            variables: HashSet::new(),
        }
    }

    pub fn clear(&mut self) {
        self.sparks.clear();
        self.variables.clear();
    }
}

#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    pub name: String,
    pub ty: Option<String>,
    pub is_mut: bool,
}

pub struct CompilerContext {
    pub depth: usize,
    pub active_targets: Vec<VariableDeclaration>,
    pub is_mutation: bool,
    pub metadata: ItemMetadata,

    // Определяет явлемся ли мы правой частью присваивания. Это нужно чтобы понять
    // относится ли вызов макроса (например spark!) к переменной или он вызван не там
    // где нужно
    pub is_right_side: bool,
    
    // Находимся ли мы в присваивании
    pub is_assign: bool,

    // Вектор ошибок компиляции чтобы накопить их
    pub compile_errors: Vec<Error>,

    // Последняя переменная найденная в local ветке
    pub variable_name: String,

    pub variable_type: String,

    // Statement это блок кода от начала до ; или фигурных скобок. Нужно точно знать
    // на каком statement мы сейчас. На старте это 0, поэтому итерацию нужно начать
    // с нуля
    pub statement_index: usize,

    pub statements: Vec<FireworkStatement>,
    pub last_statement: FireworkStatement,

    // Флаг который включается когда спарк может быть изменён, но чтобы узнать точную
    // переменную нужно пропарсить expr и залезть в path, этот флаг нужен чтобы ветка
    // path поняла что сейчас ожидается спарк на изменение и запустила проверку
    pub spark_mut_maybe: bool,

    // Переменная создана в аргументах функции или в if let
    pub is_special_var: bool,
}

impl CompilerContext {
    pub fn indent(&self) -> String {
        "  ".repeat(self.depth)
    }

    pub fn log(&self, label: &str, details: &str) {
        let targets: Vec<String> = self.active_targets
            .iter()
            .map(|v| {
                if let Some(ty) = &v.ty {
                    format!("{}: {}", v.name, ty)
                } else {
                    v.name.clone()
                }
            })
            .collect();
            
        let _targets_str = if targets.is_empty() {
            "NONE".to_string()
        } else {
            targets.join(", ")
        };
        
        println!("{}[{}] Target: [{}] | Mutation: {} | Details: {}", self.indent(), label, _targets_str, self.is_mutation, details);
    }
}

pub fn prepare_tokens(tokens: Vec<TokenTree>) -> (proc_macro2::TokenStream, Option<proc_macro2::TokenStream>) {
    let mut context = CompilerContext {
        depth: 0,
        active_targets: Vec::new(),
        is_mutation: false,
        metadata: ItemMetadata::default(),
        is_right_side: false,
        is_assign: false,
        compile_errors: Vec::new(),
        variable_name: String::from(""),
        variable_type: String::from(""),
        statement_index: 0,
        statements: Vec::new(),

        last_statement: FireworkStatement {
            action: FireworkAction::DefaultCode,
            index: 0,
        },

        spark_mut_maybe: false,
        is_special_var: false,
    };

    let token_stream: proc_macro2::TokenStream = tokens.clone().into_iter().collect();

    let parser = |input: syn::parse::ParseStream| -> syn::Result<Vec<syn::Item>> {
        let mut items = Vec::new();
        
        while !input.is_empty() {
            items.push(input.parse::<syn::Item>()?);
        }
        
        Ok(items)
    };

    let items = match syn::parse::Parser::parse2(parser, token_stream) {
        Ok(items_vec) => items_vec,
        Err(message) => {
            return (proc_macro2::TokenStream::new(), Some(message.to_compile_error()));
        },
    }; 

    let mut output = proc_macro2::TokenStream::new();
    let mut error_tokens = None;
    
    for item in items {
        output.extend(parse_items(item, &mut context));
    }

    if !context.compile_errors.is_empty() {
        let mut final_error = context.compile_errors.remove(0);
        
        for error in context.compile_errors {
            final_error.combine(error);
        }

        error_tokens = Some(final_error.to_compile_error());
    }

    println!("{:#?}", context.statements);
    
    (output, error_tokens)
}
