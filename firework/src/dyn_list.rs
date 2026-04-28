// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use core::mem;

/// Трейт который должны реализовать все скины для поддержки видимости в списках. Он
/// гарантирует наличие метода visible
pub trait SkinVisibility {
    fn visible(&self, state: bool);
    fn unmount(self);
}

/// Результат поиска элемента в списке
pub enum ListEntry<'a, K, T> {
    /// Элемент найден в старом кадре и перенесён в текущий
    Occupied(&'a mut T),

    /// Элемента с таким ключом нет и нужно вставить новый
    Vacant(VacantEntry<'a, K, T>),
}

/// Структура для вставки нового элемента
pub struct VacantEntry<'a, K, T> {
    list: &'a mut DynList<K, T>,
    key: K,
}

impl<'a, K: Eq + PartialEq, T: SkinVisibility> VacantEntry<'a, K, T> {
    /// Вставляет созданный виджет в список и возвращает мутабельную ссылку на него
    pub fn insert(self, value: T) -> &'a mut T {
        self.list.insert_at_current(self.key, value)
    }
}

#[cfg(not(feature = "no-alloc"))]
pub struct DynList<K, T> {
    current_items: Vec<(K, T)>,
    old_items: Vec<(K, T)>,
}

#[cfg(not(feature = "no-alloc"))]
impl<K: Eq + PartialEq, T: SkinVisibility> DynList<K, T> {
    pub fn new() -> Self {
        Self {
            current_items: Vec::new(),
            old_items: Vec::new(),
        }
    }

    /// Вызывается до цикла
    pub fn begin_pass(&mut self) {
        mem::swap(&mut self.current_items, &mut self.old_items);
        self.current_items.clear();
    }

    /// Поиск элемента по ключу
    pub fn entry(&mut self, key: K) -> ListEntry<'_, K, T> {
        let found_idx = self.old_items.iter()
            .position(|(k, _)| k == &key);

        if let Some(idx) = found_idx { 
            let item = self.old_items.swap_remove(idx);
            self.current_items.push(item);
            ListEntry::Occupied(&mut self.current_items.last_mut().unwrap().1)
        } else {
            ListEntry::Vacant(VacantEntry { list: self, key })
        }
    }

    /// Вспомогательный метод для VacantEntry
    fn insert_at_current(&mut self, key: K, value: T) -> &mut T {
        self.current_items.push((key, value));
        &mut self.current_items.last_mut().unwrap().1
    }

    /// Завершение прохода, убивает элементы которых больше нет в коде
    pub fn end_pass(&mut self) {
        for (_, item) in self.old_items.drain(..) {
            item.unmount();
        }
    }

    /// Групповое управление видимостью для финализатора экрана
    pub fn visible(&self, state: bool) {
        for (_, item) in &self.current_items {
            item.visible(state);
        }
    }
}

#[cfg(feature = "no-alloc")]
pub struct DynList<K, T> {
    current_items: [Option<(K, T)>; 64],
    old_items: [Option<(K, T)>; 64],
    current_count: usize,
}

#[cfg(feature = "no-alloc")]
impl<K: Eq + PartialEq, T: SkinVisibility> DynList<K, T> {
    pub fn new() -> Self {
        Self {
            current_items: [const { None }; 64],
            old_items: [const { None }; 64],
            current_count: 0,
        }
    }

    pub fn begin_pass(&mut self) {
        mem::swap(&mut self.current_items, &mut self.old_items);
        self.current_count = 0;
    }

    pub fn entry(&mut self, key: K) -> ListEntry<'_, K, T> {
        let mut found_idx = None;
        for i in 0..64 {
            if let Some((ref k, _)) = self.old_items[i] {
                if k == &key {
                    found_idx = Some(i);
                    break;
                }
            }
        }

        if let Some(idx) = found_idx {
            let item = self.old_items[idx].take().unwrap();
            let pos = self.current_count;
            self.current_items[pos] = Some(item);
            self.current_count += 1;
            ListEntry::Occupied(&mut self.current_items[pos].as_mut().unwrap().1)
        } else {
            ListEntry::Vacant(VacantEntry { list: self, key })
        }
    }

    fn insert_at_current(&mut self, key: K, value: T) -> &mut T {
        let pos = self.current_count;
        
        // На 16 битном устройстве (embedded) 16 * 2 = 32 байта, это нормально
        if pos >= 16 {
            panic!("DynList is overflow, disable no-alloc mode");
        }
        
        self.current_items[pos] = Some((key, value));
        self.current_count += 1;
        &mut self.current_items[pos].as_mut().unwrap().1
    }

    pub fn end_pass(&mut self) {
        for i in 0..16 {
            if let Some(item) = self.old_items[i].take() {
                item.unmount();
            }
        }
    }

    pub fn visible(&self, state: bool) {
        for i in 0..self.current_count {
            if let Some((_, item)) = &self.current_items[i] {
                item.visible(state);
            }
        }
    }
}
