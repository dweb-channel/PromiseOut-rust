use std::future::Future;

/// An Async Promise
///
/// A promise can be resolved or rejected, either consumes the promise. The
/// consumer is a future that can be `.await`ed.
///
/// ```
/// use promise_out::{Promise, pair::Producer};
/// use futures::executor::block_on;
/// use std::thread;
/// let (promise, consumer) = Producer::<String>::new();
///
/// let task1 = thread::spawn(move || block_on(async {
///     println!("Received {:?}",  consumer.await);
/// }));
/// promise.resolve("Hi".into());
/// task1.join().expect("The task1 thread has panicked.");
/// ```
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

#[derive(Debug)]
pub enum Error {
    ProducerDropped,
}

#[derive(Debug)]
enum WakerState {
    Fresh,
    Tainted,
}


pub mod promise_out;
pub mod pair;
pub mod poly;
