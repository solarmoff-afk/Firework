// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use proc_macro2::TokenTree;

/// Парсит вектор токенов дерева, задача метода заключается в том, чтобы утсановить
/// связь сигнала и объекта
pub fn prepare_tokens(tokens: Vec<TokenTree>, depth: usize) {
    let indent = "  ".repeat(depth);
    let mut i = 0;

    // Вектор для хранения переменных которые созданы через маркер компилятора
    // signal!(). Если переменная затеняется другим значением то нужно удалить
    // сигнал отсюда
    let mut signals: Vec<String> = Vec::new();

    // Определяет явлется ли этот токен началом строки
    let mut is_start = true;

    while i < tokens.len() {
        match &tokens[i] {
            TokenTree::Punct(punct) => {
                let punct_char = punct.as_char();
                println!("{}PUNCT: '{}'", indent, punct_char);

                // Заранее нужно установить что этот токен не начало строки чтобы
                // сделать код чище. Ниже если будет символ завершающий строку то
                // is_start автоматически станет true
                is_start = false;

                // Символы {, } и ; означают завершение строки, то есть маркер signal
                // там можно не искать. Также запятая может завершать строку, но там
                // зависит от контекста. Пока что не нужно
                if punct_char == ';' {
                    println!("{}NEW LINE: {}", indent, punct_char);
                    is_start = true;
                }

                i += 1;
            },

            TokenTree::Ident(ident) => {
                println!("{}IDENT: '{}'", indent, ident);
                
                is_start = false;
                i += 1;
            },

            TokenTree::Literal(lit) => {
                println!("{}LITERAL: '{}'", indent, lit);

                is_start = false;
                i += 1;
            }

            TokenTree::Group(group) => {
                println!("{}GROUP ({:?}) {{", indent, group.delimiter());

                // Внутри группы это уже новая строка
                is_start = true;
                println!("{}NEW LINE (START GROUP)", indent);
                
                let inner: Vec<TokenTree> = group.stream().into_iter().collect();
                prepare_tokens(inner, depth + 1);

                println!("{}}}", indent);

                // Когда группа заканчивается то это также начало новой строки
                is_start = true;
                println!("{}NEW LINE (END GROUP)", indent);

                i += 1;
            }
        } 
    }
}
