// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use futures::Stream;
use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;
use tokio::time::{self, Duration};

pub struct AsyncSpark<T> {
    stream: Option<Pin<Box<dyn Stream<Item = AsyncStatus<T>> + Unpin + Send>>>,
    status: Option<AsyncStatus<T>>,
}

impl<T> AsyncSpark<T>
where
    T: Send + 'static,
{
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
pub struct AsyncController<T> {
    tx: mpsc::UnboundedSender<AsyncStatus<T>>,
}

impl<T> AsyncController<T> {
    pub fn new(tx: mpsc::UnboundedSender<AsyncStatus<T>>) -> Self {
        AsyncController { tx }
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
