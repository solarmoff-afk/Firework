// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use proc_macro2::TokenTree;

use super::actions::{FireworkIR, FireworkStatement, FireworkAction};

pub struct CodeGen {
    pub ir: FireworkIR,
}

impl CodeGen {
    pub fn new(ir: FireworkIR) -> Self {
        Self {
            ir,
        }
    }

    pub fn run(&self) {
        for statement in self.ir.statements.iter() {
            println!("{:#?}", statement);
        }
    } 
}
