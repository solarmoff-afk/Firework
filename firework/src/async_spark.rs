// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use futures::Stream;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;
use tokio::time::{self, Duration};

pub struct AsyncSpark<T> {
    stream: Option<Pin<Box<dyn Stream<Item = AsyncStatus<T>> + Unpin + Send>>>,
    status: Option<AsyncStatus<T>>,
}

impl<T: Default + Send + 'static> AsyncSpark<T> {
    pub fn new() -> Self {
        AsyncSpark {
            stream: None,
            status: None,
        }
    }

    pub fn create<F, Fut>(&mut self, producer: F)
    where
        F: FnOnce(AsyncController<T>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send,
    {
        let (tx, rx) = mpsc::unbounded_channel();
        let _ = tx.send(AsyncStatus::Loading);

        let controller = AsyncController::new(tx);

        tokio::spawn(async move {
            producer(controller).await;
        });

        self.stream = Some(Box::pin(
            tokio_stream::wrappers::UnboundedReceiverStream::new(rx),
        ));
    }

    pub fn poll(&mut self) -> Option<AsyncStatus<T>> {
        if let Some(status) = self.status.take() {
            return Some(status);
        }

        if let Some(stream) = self.stream.as_mut() {
            let waker = futures::task::noop_waker();
            let mut cx = Context::from_waker(&waker);

            match stream.as_mut().poll_next(&mut cx) {
                Poll::Ready(Some(status)) => {
                    self.status = Some(status);
                    return self.status.take();
                }

                Poll::Ready(None) => {
                    self.stream = None;
                }

                Poll::Pending => {}
            }
        }

        None
    }
}

/// A middleware for managing asynchronous state from an asynchronous closure. It is needed to
/// avoid tying the user to a single specific runtime
pub struct AsyncController<T: Default> {
    tx: mpsc::UnboundedSender<AsyncStatus<T>>,

    // Изначально это значение дефолтное, но потом оно используется как прокси для того чтобы
    // при следующем DerefMut или Drop забрать это значение, сделать отправку и уже вернуть
    // мутабельную ссылку снова на дефолтное значение
    value: T,

    // Флаг для определения были ли другие DerefMut для этого (является ли текущее значение
    // первым)
    is_first: bool,
}

impl<T: Default> AsyncController<T> {
    pub fn new(tx: mpsc::UnboundedSender<AsyncStatus<T>>) -> Self {
        Self {
            tx,
            value: T::default(),
            is_first: true,
        }
    }

    pub fn send(&self, value: T) {
        let _ = self.tx.send(AsyncStatus::Ready(value));
    }

    pub fn error(&self) {
        let _ = self.tx.send(AsyncStatus::Error(AsyncError::new()));
    }

    pub fn error_message(&self, msg: impl Into<String>) {
        let _ = self
            .tx
            .send(AsyncStatus::Error(AsyncError::with_message(msg)));
    }

    pub async fn sleep_s(&self, secs: u64) {
        time::sleep(Duration::from_secs(secs)).await;
    }

    pub async fn sleep_ms(&self, ms: u64) {
        time::sleep(Duration::from_millis(ms)).await;
    }
}

impl<T: Default> Deref for AsyncController<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: Default> DerefMut for AsyncController<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Если это значение не первое (не дефольное) то оно забирается через тейк чтобы
        // на его месте было уже дефолтное значение, а это используется для отправки через
        // канал
        if !self.is_first {
            let old = std::mem::take(&mut self.value);
            let _ = self.tx.send(AsyncStatus::Ready(old));
        }

        self.is_first = false;
        &mut self.value
    }
}

impl<T: Default> Drop for AsyncController<T> {
    fn drop(&mut self) {
        // При Drop если пользователь использовал DerefMut происходит отправка текущего
        // значения потому-что
        //  - DerefMut, пропуск из-за is_first, теперь тут 10
        //  - DerefMut, 10 отправляется через канал, теперь тут 20
        //  - Drop, всё ещё 20 которая не была отправлена, но благодаря этой логике тут
        //    отправка произойдёт если был deref_mut
        if self.is_first {
            let _ = self
                .tx
                .send(AsyncStatus::Ready(std::mem::take(&mut self.value)));
        }
    }
}

pub enum AsyncStatus<T> {
    Ready(T),
    Error(AsyncError),
    Loading,
}

pub struct AsyncError {
    message: Option<String>,
}

impl AsyncError {
    pub fn new() -> Self {
        AsyncError { message: None }
    }

    pub fn with_message(msg: impl Into<String>) -> Self {
        AsyncError {
            message: Some(msg.into()),
        }
    }
}

impl fmt::Display for AsyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.message {
            Some(msg) => write!(f, "{}", msg),
            None => write!(f, "AsyncError"),
        }
    }
}
