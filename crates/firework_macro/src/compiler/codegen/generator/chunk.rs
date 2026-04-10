// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0. Copyright (c) 2026 Firework

use proc_macro2::{TokenStream, TokenTree, Span, Group};

use std::collections::HashMap;

/// Контйнер для хранения строк с привязкой к спанам. Спан это место в исходном файле
/// где находится эта строка которая была взята из стейтемента
#[derive(Clone)]
pub struct ChunkStore {
    // Строка -> Спан
    pub chunks: Vec<(String, Span)>,
    
    // Текущий спан. Это нужно так как раньше весь код работал с String и переписывать
    // все вызовы push_str слишком долго, поэтому вместо этого задаётся спан который
    // будет задан всем строкам которые будут созданы далее
    current_span: Span,
}

impl ChunkStore {
    pub fn new() -> Self {
        Self {
            chunks: Vec::new(),
            current_span: Span::call_site(),
        }
    }

    /// Установить спан для всех следующих строк
    pub fn set_span(&mut self, span: Span) {
        self.current_span = span;
    }

    /// Добавить строку. Если не сделать set_span то спан будет указывать на строку
    /// вызова макроса ui!
    pub fn push_str(&mut self, string: &str) {
        self.chunks.push((string.to_string(), self.current_span));
    }

    /// Метод для объединения чанков
    pub fn extend(&mut self, other: &ChunkStore) {
        self.chunks.extend(other.chunks.clone());
    }

    /// Преобразование чанка в строку. Работает только если есть фича debug_output
    /// (в firework_ui она называется detail)
    #[cfg(feature = "debug_output")]
    pub fn to_string(&self) -> String {
        self.chunks.iter().map(
            |(string, _)| string.as_str()).collect()
    }

    /// Метод для преобразования чанка в TokenStreen для инлайна на место макроса
    pub fn generate_code(&self) -> TokenStream {
        let mut full_code = String::new();
        let mut span_map = HashMap::new();

        for (idx, (text, span)) in self.chunks.iter().enumerate() {
            let marker_id = format!("__fw_span_idx_{}__", idx);
            
            // Добавление маркера и пробела чтобы токены не слиплись
            full_code.push_str(" ");
            full_code.push_str(&marker_id);
            full_code.push_str(" ");
            full_code.push_str(text);
            
            span_map.insert(marker_id, *span);
        }

        // Парсинг всей строки целиком
        let ts: TokenStream = full_code.parse().expect("Failed to parse generated code");

        let mut current_active_span = Span::call_site();
        self.apply_spans_and_filter(ts, &span_map, &mut current_active_span)
    }

    fn apply_spans_and_filter(
        &self, 
        token_stream: TokenStream, 
        span_map: &HashMap<String, Span>,
        current_span: &mut Span
    ) -> TokenStream {
        let mut output = TokenStream::new();

        for token in token_stream {
            match token {
                TokenTree::Group(group) => {
                    let inner = self.apply_spans_and_filter(group.stream(), span_map, current_span);
                    let mut new_group = Group::new(group.delimiter(), inner);
                    new_group.set_span(*current_span);

                    output.extend(std::iter::once(TokenTree::Group(new_group)));
                },

                TokenTree::Ident(id) => {
                    let name = id.to_string();

                    if let Some(new_span) = span_map.get(&name) {
                        *current_span = *new_span;
                    } else {
                        let mut new_id = id.clone();
                        new_id.set_span(*current_span);
                        output.extend(std::iter::once(TokenTree::Ident(new_id)));
                    }
                },

                mut other => {
                    other.set_span(*current_span);
                    output.extend(std::iter::once(other));
                },
            }
        }

        output
    }
}
