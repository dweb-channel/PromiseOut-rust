use std::{
    future::Future, task::{Poll, Waker},
    sync::{
        mpsc::{channel, Receiver, RecvError, TryRecvError, Sender},
        Arc, Mutex,
    }
};
use crate::{Promise, Error};
#[derive(Debug, Clone)]
pub struct Producer<T> {
    sender: Sender<T>,
}

#[derive(Debug)]
pub struct Consumer<T> {
    receiver: Receiver<T>
}

impl<T> Future for Consumer<T> {
    type Output = Result<T, Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.receiver.try_recv() {
            Ok(value) => Poll::Ready(Ok(value)),
            Err(TryRecvError::Empty) => Poll::Pending,
            Err(TryRecvError::Disconnected) => Poll::Ready(Err(Error::ProducerDropped))
        }
    }
}

// impl<T> Clone for Producer<T> {
//     fn clone(&self) -> Self {
//         Self {
//             sender: self.sender.clone(),
//             receiver: self.receiver.clone(),
//         }
//     }
// }

// impl<T> Default for Producer<T> {
//     fn default() -> Self {
//         let (tx, rx) = channel();
//         Self {
//             sender: tx,
//             receiver: rx,
//         }
//     }
// }

impl<T> Promise<T> for Producer<T> {
    type Waiter = Consumer<T>;
    fn resolve(self, value: T) {
        self.sender.send(value).unwrap();
    }

    fn new() -> (Self, Self::Waiter) where Self: Sized{
        let (tx, rx) = channel();
        (Self {sender: tx },
         Consumer { receiver: rx })
    }
}
