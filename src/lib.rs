
#![doc = include_str!("../README.md")]
use std::future::Future;

pub trait Promise<T> {
    type Waiter : Future;

    /// Resolve the promise's value.
    ///
    /// Typically a promise library will also offer a `reject()` method to serve
    /// a user specified error. However, returning an `Err(E)` for a
    /// `Promise<Result<T,E>>` serves the same purpose.
    fn resolve(self, value: T);
    // fn reject(self, value: Error);
    fn new() -> (Self, Self::Waiter) where Self: Sized;
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
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
