use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::{future::Future, task::{Poll, Waker}};
use crate::{Promise, Error};

/// This `pair::Producer` promise can only have one consumer. The consumer
/// returns a `Result<T,E>`.
///
/// # Examples
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
#[derive(Debug)]
pub struct Producer<T> {
    promise: Arc<Mutex<Inner<T>>>,
}

#[derive(Debug)]
pub struct Consumer<T> {
    promise: Arc<Mutex<Inner<T>>>,
}

#[derive(Debug)]
struct Inner<T> {
    value: Option<Result<T,Error>>,
    waker: Option<Waker>,
}

impl<T> Promise<T> for Producer<T> {
    // type Output = T;
    // type Error = E;
    type Waiter = Consumer<T>;
    #[allow(dead_code)]
    ///promiseOut.resolve
    ///
    /// # Examples
    ///
    /// ```
    /// use promise_out::pair::Producer;
    /// use promise_out::Promise;
    /// use futures::executor::block_on;
    /// use std::thread;
    /// let (op, op_a) = Producer::<String, String>::new();
    /// let task1 = thread::spawn(move || block_on(async {
    ///     println!("我等到了{:?}",  op_a.await);
    /// }));
    /// let task2 = thread::spawn(move || block_on(async {
    ///     println!("我发送了{:?}", op.resolve(String::from("🍓")));
    /// }));
    /// task1.join().expect("The task1 thread has panicked");
    /// task2.join().expect("The task2 thread has panicked");
    /// ```
    fn resolve(self, value: T) {
        let mut promise = self.promise.lock().unwrap();
        promise.value = Some(Ok(value));
        if let Some(waker) = promise.waker.take() {
            waker.wake()
        }
    }
    ///promiseOut.reject
    ///
    /// # Examples
    ///
    /// ```
    /// use promise_out::pair::Producer;
    /// use promise_out::Promise;
    /// use futures::executor::block_on;
    /// use std::thread;
    /// let (op, op_a) = Producer::<String, String>::new();
    /// let task1 = thread::spawn(move || block_on(async {
    ///     println!("我等到了{:?}",  op_a.await);
    /// }));
    /// let task2 = thread::spawn(move || block_on(async {
    ///     println!("我发送了{:?}", op.reject(String::from("💥")));
    /// }));
    /// task1.join().expect("The task1 thread has panicked");
    /// task2.join().expect("The task2 thread has panicked");
    /// ```
    // #[allow(dead_code)]
    // fn reject(self, err: Error) {
    //     let mut promise = self.promise.lock().unwrap();
    //     promise.value = Some(Err(err));
    //     if let Some(waker) = promise.waker.take() {
    //         waker.wake()
    //     }
    // }
    fn new() -> (Self, Consumer<T>) {
        let inner = Arc::new(Mutex::new(Inner {
                value: None,
                waker: None,
            }));
        (Self { promise: inner.clone() }, Consumer { promise: inner })
    }
}

impl<T> Drop for Producer<T> {
    fn drop(&mut self) {
        let mut promise = self.promise.lock().unwrap();
        if let Some(waker) = promise.waker.take() {
            promise.value = Some(Err(Error::ProducerDropped));
            waker.wake()
        }
    }
}

impl<T> Future for Consumer<T> {
    type Output = Result<T, Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut promise = self.promise.lock().unwrap();
        match promise.value.take() {
            Some(value) => Poll::Ready(value),
            None => {
                promise.waker.replace(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

#[cfg(test)]
mod tests {
#[allow(unused_imports)]
use futures::executor::block_on;
#[allow(unused_imports)]
use std::thread;
use super::Producer;
use crate::Promise;

#[allow(unused_must_use)]
#[test]
fn test_promise_out_resolve() {
    let (op, op_a) = Producer::<String>::new();
    let task1 = thread::spawn(move || {
        block_on(async {
            println!("我等到了{:?}", op_a.await.unwrap());
        })
    });
    let task2 = thread::spawn(move || {
        block_on(async {
            println!("我发送了了{:?}", op.resolve(String::from("🍓")));
        })
    });
    task1.join().expect("The task1 thread has panicked");
    task2.join().expect("The task2 thread has panicked");
}

#[allow(unused_must_use)]
#[should_panic(expected = "The task1 thread has panicked")]
#[test]
fn test_promise_out_unresolved() {
    let (op, op_a) = Producer::<String>::new();
    let task1 = thread::spawn(move || {
        block_on(async {
            println!("我等到了{:?}", op_a.await.unwrap());
        })
    });
    let task2 = thread::spawn(move || {
        block_on(async {
            // Ensure we move the producer into this thread.
            let _op = op;
            // println!("我发送了了{:?}", op.resolve(String::from("🍓")));
        })
    });
    task1.join().expect("The task1 thread has panicked");
    task2.join().expect("The task2 thread has panicked");
}

#[allow(unused_must_use)]
#[test]
fn test_promise_out_no_consumer() {
    let (op, op_a) = Producer::<String>::new();
    let task1 = thread::spawn(move || {
        block_on(async {
            let _op_a = op_a;
            // println!("我等到了{:?}", op_a.await.unwrap());
        })
    });
    let task2 = thread::spawn(move || {
        block_on(async {
            // Ensure we move the producer into this thread.
            println!("我发送了了{:?}", op.resolve(String::from("🍓")));
        })
    });
    task1.join().expect("The task1 thread has panicked");
    task2.join().expect("The task2 thread has panicked");
}

#[test]
fn test_promise_out_reject() {
    let (a, b) = Producer::<Result<String, String>>::new();
    let task1 = thread::spawn(|| {
        block_on(async {
            println!("我等到了{:?}", b.await);
        })
    });
    let task2 = thread::spawn(|| {
        block_on(async {
            println!("我发送了了{:?}", a.resolve(Err("reject!!".into())));
        })
    });
    task1.join().expect("The task1 thread has panicked");
    task2.join().expect("The task2 thread has panicked");
}
}
