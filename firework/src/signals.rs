use std::fmt::Debug;
use std::marker::PhantomData;

#[derive(Clone, Copy)]
pub struct ReadSignal<T: Copy> {
    value: T,
    _phantom: PhantomData<T>,
}

pub struct WriteSignal<T: Copy> {
    _phantom: PhantomData<T>,
}

pub fn create_signal<T: Copy + Debug>(initial_value: T) -> (ReadSignal<T>, WriteSignal<T>) {
    println!("Сигнал создан со значением {:?}", initial_value);
    (
        ReadSignal { value: initial_value, _phantom: PhantomData },
        WriteSignal { _phantom: PhantomData },
    )
}

impl<T: Copy> ReadSignal<T> {
    pub fn get(&self) -> T {
        self.value
    }
}

impl<T: Copy + Debug> WriteSignal<T> {
    pub fn set(&self, new_value: T) {
        println!("Попытка установить значение сигнала на {:?}", new_value);
    }
}