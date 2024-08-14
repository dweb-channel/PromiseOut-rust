//! A channel promise uses a multi-producer, single-consumer channel as its
//! backend. This allows for the Producer to be cloned but not the Consumer.
//!
use crate::{Error, Promise, WakerState};
use std::{
    future::Future,
    sync::{
        mpsc::{channel, Receiver, Sender, TryRecvError},
        Arc, Mutex,
    },
    task::{Poll, Waker},
};
#[derive(Debug, Clone)]
pub struct Producer<T> {
    sender: Sender<T>,
    promise: Arc<Mutex<Inner>>,
}

#[derive(Debug)]
pub struct Consumer<T> {
    receiver: Receiver<T>,
    promise: Arc<Mutex<Inner>>,
}

#[derive(Debug)]
struct Inner {
    waker: Result<Waker, WakerState>,
}

impl<T> Future for Consumer<T> {
    type Output = Result<T, Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.receiver.try_recv() {
            Ok(value) => Poll::Ready(Ok(value)),
            Err(TryRecvError::Empty) => {
                let mut promise = self.promise.lock().unwrap();
                match std::mem::replace(&mut promise.waker, Ok(cx.waker().clone())) {
                    Err(WakerState::Tainted) => Poll::Ready(Err(Error::ProducerDropped)),
                    _ => Poll::Pending,
                }
            }
            Err(TryRecvError::Disconnected) => Poll::Ready(Err(Error::ProducerDropped)),
        }
    }
}

impl<T> Promise<T> for Producer<T> {
    type Waiter = Consumer<T>;
    fn resolve(self, value: T) {
        self.sender.send(value).unwrap();
        let mut promise = self.promise.lock().unwrap();
        if let Ok(waker) = std::mem::replace(&mut promise.waker, Err(WakerState::Tainted)) {
            waker.wake()
        }
    }

    fn new() -> (Self, Self::Waiter)
    where
        Self: Sized,
    {
        let (tx, rx) = channel();
        let inner = Arc::new(Mutex::new(Inner {
            waker: Err(WakerState::Fresh),
        }));
        (
            Producer {
                sender: tx,
                promise: inner.clone(),
            },
            Consumer {
                receiver: rx,
                promise: inner,
            },
        )
    }
}
