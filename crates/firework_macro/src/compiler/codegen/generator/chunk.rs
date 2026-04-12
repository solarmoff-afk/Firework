// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0. Copyright (c) 2026 Firework

use proc_macro2::{TokenStream, Span};
use quote::quote;

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
        let code: String = self.chunks.iter().map(|(s, _)| s.as_str()).collect();
        
        match code.parse() {
            Ok(ts) => ts,
            Err(_) => {
                let fallback = quote! {
                    ::core::compile_error!(concat!("Failed to parse generated code: ", #code));
                };

                fallback
            }
        }
    }
}
