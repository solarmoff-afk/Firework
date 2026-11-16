// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use core::mem;

use crate::BuildContext;
use crate::std_widgets::widget::Widget;

/// Результат поиска элемента в списке
pub enum ListEntry<'a, K, T, const N: usize = 64> {
    /// Элемент найден в старом кадре и перенесён в текущий
    Occupied(&'a mut T),

    /// Элемента с таким ключом нет и нужно вставить новый
    Vacant(VacantEntry<'a, K, T, N>),
}

/// Структура для вставки нового элемента
pub struct VacantEntry<'a, K, T, const N: usize = 64> {
    list: &'a mut DynList<K, T, N>,
    key: K,
}

impl<'a, K: Eq + PartialEq, T: Widget, const N: usize> VacantEntry<'a, K, T, N> {
    /// Вставляет созданный виджет в список и возвращает мутабельную ссылку на него
    pub fn insert(self, value: T) -> &'a mut T {
        self.list.insert_at_current(self.key, value)
    }
}

#[cfg(not(feature = "no-alloc"))]
pub struct DynList<K, T, const N: usize = 64> {
    current_items: Vec<(K, T)>,
    old_items: Vec<(K, T)>,
    _marker: core::marker::PhantomData<[(); N]>,
}

#[cfg(not(feature = "no-alloc"))]
impl<K: Eq + PartialEq, T: Widget, const N: usize> Default for DynList<K, T, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "no-alloc"))]
impl<K: Eq + PartialEq, T: Widget, const N: usize> DynList<K, T, N> {
    pub fn new() -> Self {
        Self {
            current_items: Vec::with_capacity(N),
            old_items: Vec::with_capacity(N),
            _marker: core::marker::PhantomData,
        }
    }

    /// Вызывается до цикла
    pub fn begin_pass(&mut self) {
        mem::swap(&mut self.current_items, &mut self.old_items);
        self.current_items.clear();
    }

    /// Поиск элемента по ключу
    pub fn entry(&mut self, key: K) -> ListEntry<'_, K, T, N> {
        let found_idx = self.old_items.iter().position(|(k, _)| k == &key);

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
    pub fn visible(&mut self, state: bool) {
        for (_, item) in &mut self.current_items {
            item.visible(state);
        }
    }

    pub fn len(&self) -> i16 {
        // TODO: Придумать что-то чтобы не было паники
        self.current_items
            .len()
            .try_into()
            .expect("List len >i32::MAX")
    }

    /// Устанавливает z координату в диапазоне. Нужно вызывать после end_pass
    pub fn set_z_range(
        &mut self,
        /* z координата первого элемента списка, для каждого следующего элемента
         * делается +1 к этому параметру. Так продолжается пока элементы не подойдут
         * к концу */
        z_start: i16,
    ) {
        let mut z = z_start;

        for (_, item) in self.current_items.iter_mut() {
            item.set_z(z);
            z = z.saturating_add(1);
        }
    }

    pub fn __event(&mut self, context: BuildContext) {
        for (_, item) in &mut self.current_items {
            item.__event(context);
        }
    }

    pub fn __tick(&mut self, delta: u32) -> bool {
        let mut any = false;

        for (_, item) in &mut self.current_items {
            any |= item.__tick(delta);
        }

        any
    }
}

#[cfg(feature = "no-alloc")]
pub struct DynList<K, T, const N: usize = 64> {
    current_items: [Option<(K, T)>; N],
    old_items: [Option<(K, T)>; N],
    current_count: usize,
}

#[cfg(feature = "no-alloc")]
impl<K: Eq + PartialEq, T: Widget, const N: usize> Default for DynList<K, T, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "no-alloc")]
impl<K: Eq + PartialEq, T: Widget, const N: usize> DynList<K, T, N> {
    pub fn new() -> Self {
        Self {
            current_items: [const { None }; N],
            old_items: [const { None }; N],
            current_count: 0,
        }
    }

    pub fn begin_pass(&mut self) {
        mem::swap(&mut self.current_items, &mut self.old_items);
        self.current_count = 0;
    }

    pub fn entry(&mut self, key: K) -> ListEntry<'_, K, T, N> {
        let mut found_idx = None;
        for i in 0..N {
            if let Some((ref k, _)) = self.old_items[i] {
                if k == &key {
                    found_idx = Some(i);
                    break;
                }
            }
        }

        if let Some(idx) = found_idx {
            let pos = self.current_count;

            if pos >= N {
                panic!("DynList is overflow, disable no-alloc mode");
            }

            let item = self.old_items[idx].take().unwrap();
            self.current_items[pos] = Some(item);
            self.current_count += 1;
            ListEntry::Occupied(&mut self.current_items[pos].as_mut().unwrap().1)
        } else {
            ListEntry::Vacant(VacantEntry { list: self, key })
        }
    }

    fn insert_at_current(&mut self, key: K, value: T) -> &mut T {
        let pos = self.current_count;

        if pos >= N {
            panic!("DynList is overflow, disable no-alloc mode");
        }

        self.current_items[pos] = Some((key, value));
        self.current_count += 1;
        &mut self.current_items[pos].as_mut().unwrap().1
    }

    pub fn end_pass(&mut self) {
        for i in 0..N {
            if let Some((_, item)) = self.old_items[i].take() {
                item.unmount();
            }
        }
    }

    pub fn visible(&mut self, state: bool) {
        for i in 0..self.current_count {
            if let Some((_, item)) = &mut self.current_items[i] {
                item.visible(state);
            }
        }
    }

    pub fn __event(&mut self, context: BuildContext) {
        for i in 0..self.current_count {
            if let Some((_, item)) = &mut self.current_items[i] {
                item.__event(context);
            }
        }
    }

    pub fn __tick(&mut self, delta: u32) -> bool {
        let mut any = false;
        for i in 0..self.current_count {
            if let Some((_, item)) = &mut self.current_items[i] {
                any |= item.__tick(delta);
            }
        }
        any
    }

    pub fn len(&self) -> i16 {
        // Паники не будет, так как в no-alloc длина списка всегда 64
        self.current_count.try_into().unwrap()
    }

    pub fn set_z_range(&mut self, z_start: i16) {
        let mut current_z = z_start;

        for i in 0..self.current_count {
            if let Some((_, item)) = &mut self.current_items[i] {
                item.set_z(current_z);
                current_z = current_z.saturating_add(1);
            }
        }
    }
}

impl<K: Eq + PartialEq, T: Widget, const N: usize> Widget for DynList<K, T, N> {
    fn position(&mut self, _position: (i32, i32)) {}

    fn visible(&mut self, visible: bool) {
        self.visible(visible);
    }

    fn unmount(mut self) {
        self.end_pass();

        #[cfg(not(feature = "no-alloc"))]
        for (_, item) in self.current_items {
            item.unmount();
        }

        #[cfg(feature = "no-alloc")]
        for i in 0..self.current_count {
            if let Some((_, item)) = self.current_items[i].take() {
                item.unmount();
            }
        }
    }

    fn layout(&mut self, _constraints: crate::layout::Constraints) -> crate::layout::Size {
        crate::layout::Size {
            width: 0,
            height: 0,
        }
    }

    fn get_z_size(&mut self) -> i16 {
        self.len()
    }

    fn set_z(&mut self, z: i16) {
        self.set_z_range(z);
    }

    fn set_vcanvas(&self, vcanvas_handle: usize) {
        #[cfg(not(feature = "no-alloc"))]
        for (_, item) in &self.current_items {
            item.set_vcanvas(vcanvas_handle);
        }

        #[cfg(feature = "no-alloc")]
        for i in 0..self.current_count {
            if let Some((_, item)) = &self.current_items[i] {
                item.set_vcanvas(vcanvas_handle);
            }
        }
    }

    fn __event(&mut self, context: BuildContext) {
        self.__event(context);
    }

    fn __tick(&mut self, delta: u32) -> bool {
        self.__tick(delta)
    }
}
