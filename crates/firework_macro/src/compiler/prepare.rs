// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use proc_macro2::TokenTree;

pub fn prepare_tokens(tokens: Vec<TokenTree>, depth: usize) {
    let indent = "  ".repeat(depth);
    let mut i = 0;
    
    while i < tokens.len() {
        match &tokens[i] {
            TokenTree::Ident(ident) => {
                println!("{}IDENT: '{}'", indent, ident);
                i += 1;
            }
            
            TokenTree::Punct(punct) => {
                println!("{}PUNCT: '{}'", indent, punct.as_char());
                i += 1;
            }
            
            TokenTree::Literal(lit) => {
                println!("{}LITERAL: '{}'", indent, lit);
                i += 1;
            }

            TokenTree::Group(group) => {
                println!("{}GROUP ({:?}) {{", indent, group.delimiter());
                
                let inner: Vec<TokenTree> = group.stream().into_iter().collect();
                prepare_tokens(inner, depth + 1);

                println!("{}}}", indent);
                
                i += 1;
            }
        }
    }
}
