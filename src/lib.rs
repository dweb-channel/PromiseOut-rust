#![doc = include_str!("../README.md")]
use std::future::Future;
use thiserror::Error;

/// The trait for a promise.
pub trait Promise<T> {
    type Waiter: Future;

    /// Resolve the promise's value.
    ///
    /// Typically a promise library will also offer a `reject()` method to serve
    /// a user specified error. However, returning an `Err(E)` for a
    /// `Promise<Result<T,E>>` serves the same purpose.
    fn resolve(self, value: T);

    /// Return a (producer, consumer) pair.
    fn new() -> (Self, Self::Waiter)
    where
        Self: Sized;
}

#[derive(Debug, PartialEq, Eq, Error)]
pub enum Error {
    #[error("producer dropped")]
    ProducerDropped,
}

#[derive(Debug)]
enum WakerState {
    Fresh,
    Tainted,
}

pub mod channel;
pub mod pair;
pub mod poly;
