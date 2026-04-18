// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

pub mod traits;
mod visitor;
mod reactive;
mod flash_pass;

use syn::visit_mut::VisitMut;
use syn::spanned::Spanned;
use syn::*;
use syn::visit_mut::*;
use std::collections::HashMap;
use quote::format_ident;
use proc_macro2::TokenStream;

use super::generator::static_gen::*;
use super::generator::bitmask_gen::*;
use super::consts::{CHECK_EVENT, SET_FOCUS};
use super::actions::{FireworkIR, FireworkStatement};
use super::code_builder::CodeBuilder;

pub struct CodegenVisitor<'a> {
    // IR от анализатора, содержит плоские семантические метки для каждого стейтемента,
    // а также содержит снапшот (Мапинг спан -> метка стейтемента)
    pub ir: &'a mut FireworkIR,

    // При каждом входе в функцию проверяется есть ли запись о ней в ir, если есть то
    // этот флаг поднимается, если нет то отпускается. Если он поднят то нужно генерировать
    // код для UI, если нет то это обычная функция. Содержит внутри айди экрана который
    // используется в структуре и экземпляре которые создаются
    pub ui_id: Option<u128>,

    pub builder: CodeBuilder,
    
    pub mask_count: HashMap<u128, u8>,
}

impl<'a> CodegenVisitor<'a> {
    pub fn new(ir: &'a mut FireworkIR) -> Self {
        Self {
            ir,
            ui_id: None,
            builder: CodeBuilder::new(),
            mask_count: HashMap::new(),
        }
    }

    pub fn generate_code(&mut self, stmt: &Stmt, statements: &[FireworkStatement], body: TokenStream) -> TokenStream {
        self.find_mask_counts();
        self.builder.build(stmt, statements, body)
    }

    /// Проходится по всем экранам и вычисляет сколько нужно битовых масок для отслеживания
    /// реактивеости, по 64 спарка на одну битовую маску
    pub(crate) fn find_mask_counts(&mut self) {
        for (_screen_name, _screen_signature, screen_id) in self.ir.screens.iter() {
            // Вычисление количества битовых масок, одна битовая маска это 64 бита
            let spark_count = self.ir.screen_sparks.get(screen_id).unwrap_or(&0usize);

            // Расчёт сколько нужно битовых масок на основе количество спарков
            // 1 -> 1, 19 -> 1, 64 -> 1, 67 -> 2, 98 -> 2, 128 -> 2, 136 -> 3
            self.mask_count.insert(*screen_id, get_spark_mask(*spark_count)); 
        }
    } 
}

impl<'a> VisitMut for CodegenVisitor<'a> {
    fn visit_file_mut(&mut self, i: &mut File) {
        self.analyze_file_mut(i);
    }

    fn visit_block_mut(&mut self, i: &mut Block) {
        self.analyze_block_mut(i);
    } 
}
