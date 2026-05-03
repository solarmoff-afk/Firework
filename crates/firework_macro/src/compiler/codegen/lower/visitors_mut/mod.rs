// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

mod block;
mod item;

use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use syn::visit_mut::VisitMut;
use syn::*;

use crate::CompileFlags;
use crate::compiler::codegen::code_builder::CodeBuilder;
use crate::compiler::codegen::ir::FireworkIR;

pub struct LowerVisitor<'a> {
    // IR от анализатора, содержит плоские семантические метки для каждого стейтемента,
    // а также содержит снапшот (Мапинг спан -> метка стейтемента)
    pub ir: &'a mut FireworkIR,

    pub ui_id: Option<u128>,
    pub builder: CodeBuilder,
    pub pending_drops: Vec<(Span, TokenStream)>,
}

impl<'a> LowerVisitor<'a> {
    pub fn new(ir: &'a mut FireworkIR, flags: CompileFlags) -> Self {
        Self {
            builder: CodeBuilder::new(ir.clone(), flags),
            ir,
            ui_id: Some(0),
            pending_drops: Vec::new(),
        }
    }
}

impl<'a> VisitMut for LowerVisitor<'a> {
    fn visit_file_mut(&mut self, i: &mut File) {
        self.lower_file_mut(i);
    }

    fn visit_block_mut(&mut self, i: &mut Block) {
        self.lower_block_mut(i);
    }

    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        let old_drops = std::mem::take(&mut self.pending_drops);

        syn::visit_mut::visit_stmt_mut(self, stmt);

        if !self.pending_drops.is_empty() {
            let drops = std::mem::take(&mut self.pending_drops);

            let mut all_drops = TokenStream::new();
            for (_, tokens) in drops {
                all_drops.extend(tokens);
            }

            let new_stmt_tokens = quote! {{
                #stmt
                #all_drops
            }};

            *stmt = syn::parse2(new_stmt_tokens).unwrap();
        }

        self.pending_drops = old_drops;
    }
}
