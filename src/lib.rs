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
/// let (promise, consumer) = Producer::<String, String>::new();
///
/// let task1 = thread::spawn(move || block_on(async {
///     println!("Received {:?}",  consumer.await);
/// }));
/// promise.resolve("Hi".into());
/// task1.join().expect("The task1 thread has panicked.");
/// ```
pub trait Promise {
    type Output;
    type Error;
    type Waiter : Future;

    fn resolve(self, value: Self::Output);
    fn reject(self, err: Self::Error);
    fn new() -> (Self, Self::Waiter) where Self: Sized;
}

pub mod promise_out;
pub mod pair;
pub mod poly;
