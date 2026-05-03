// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use quote::format_ident;
use proc_macro2::Ident;

/// Это специальная структура которая позволяет оптимизировать парсинг (например format_ident)
/// благодаря сохранению прошлых результатов работы
pub struct CodeBuilderCache {
    // Так как индексы битовых масок идут последовательно можно закэшировать их в векторе, номер
    // битвой маски это индекс в векторе где хранится иденты
    widget_bitmask_idents: Vec<Ident>,
}

impl CodeBuilderCache {
    pub fn new() -> Self {
        Self {
            widget_bitmask_idents: Vec::new(),
        }
    }

    pub fn cache_widget_bitmask(&mut self, id: u8) -> &Ident {
        let id_usize = id as usize;

        // Если такой маски нет в кэше то она создаётся, на множство проверок только один
        // вызов лексера
        while self.widget_bitmask_idents.len() <= id_usize {
            let next_id = self.widget_bitmask_idents.len() as u8;
            let mask_name = format_ident!("_fwc_widget_bitmask{}", next_id);
            self.widget_bitmask_idents.push(mask_name);
        }

        &self.widget_bitmask_idents[id_usize]
    }
}
