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

    pub fn run(&self, tokens: Vec<TokenTree>) {
        self.analyze(tokens, 0);
    }

    fn analyze(&self, tokens: Vec<TokenTree>, depth: usize) {
        let indent = "  ".repeat(depth);
        let mut i = 0;
    
        while i < tokens.len() {
            match &tokens[i] {
                TokenTree::Ident(ident) => {
                    println!("{}IDENT: '{}'", indent, ident);
                    i += 1;
                },

                TokenTree::Punct(punct) => {
                    println!("{}PUNCT: '{}'", indent, punct.as_char());
                    i += 1;
                },

                TokenTree::Literal(lit) => {
                    println!("{}LITERAL: '{}'", indent, lit);
                    i += 1;
                },
                
                TokenTree::Group(group) => {
                    println!("{}GROUP ({:?}) {{", indent, group.delimiter());
                    let inner: Vec<TokenTree> = group.stream().into_iter().collect();
                    self.analyze(inner, depth + 1);
                    println!("{}}}", indent);
                    i += 1;
                },
            }
        }
    }
}
