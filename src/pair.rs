use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::{future::Future, task::{Poll, Waker}};
use crate::{Promise, Error};

/// This `pair::Producer` promise can only have one consumer. The consumer
/// returns a `Result<T,Error>`. An error is returned if the only producer has
/// been dropped; thus it will never resolve the promise.
///
/// # Examples
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
#[derive(Debug)]
pub struct Producer<T> {
    promise: Arc<Mutex<Inner<T>>>,
}

#[derive(Debug)]
pub struct Consumer<T> {
    promise: Arc<Mutex<Inner<T>>>,
}

#[derive(Debug)]
enum WakerState {
    Fresh,
    Tainted,
}

#[derive(Debug)]
struct Inner<T> {
    value: Option<T>,
    waker: Result<Waker, WakerState>,
}

impl<T> Promise<T> for Producer<T> {
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
    /// let (op, op_a) = Producer::<String>::new();
    /// let task1 = thread::spawn(move || block_on(async {
    ///     println!("我等到了{:?}",  op_a.await.unwrap());
    /// }));
    /// let task2 = thread::spawn(move || block_on(async {
    ///     println!("我发送了{:?}", op.resolve(String::from("🍓")));
    /// }));
    /// task1.join().expect("The task1 thread has panicked");
    /// task2.join().expect("The task2 thread has panicked");
    /// ```
    fn resolve(self, value: T) {
        let mut promise = self.promise.lock().unwrap();
        promise.value = Some(value);
        if let Ok(waker) = std::mem::replace(&mut promise.waker, Err(WakerState::Tainted)) {
            waker.wake()
        }
    }

    fn new() -> (Self, Consumer<T>) {
        let inner = Arc::new(Mutex::new(Inner {
                value: None,
                waker: Err(WakerState::Fresh),
            }));
        (Self { promise: inner.clone() }, Consumer { promise: inner })
    }
}

impl<T> Drop for Producer<T> {
    /// If this is an unresolved producer, wake with an error.
    fn drop(&mut self) {
        let mut promise = self.promise.lock().unwrap();
        if let Ok(waker) = std::mem::replace(&mut promise.waker, Err(WakerState::Tainted)) {
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
            Some(value) => Poll::Ready(Ok(value)),
            None => {
                match std::mem::replace(&mut promise.waker, Ok(cx.waker().clone())) {
                    Err(WakerState::Tainted) => Poll::Ready(Err(Error::ProducerDropped)),
                    _ => Poll::Pending
                }
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
            // Ensure we move the producer into this thread but we never resolve
            // it.
            // let _op = op;
            std::mem::drop(op);
            // println!("我发送了了{:?}", op.resolve(String::from("🍓")));
        })
    });
    task2.join().expect("The task2 thread has panicked");
    task1.join().expect("The task1 thread has panicked");
    // assert!(task1.join().is_err());
}

#[allow(unused_must_use)]
#[should_panic(expected = "The task1 thread has panicked")]
#[test]
fn test_promise_out_unresolved_consumer_first() {
    let (op, op_a) = Producer::<String>::new();
    let task1 = thread::spawn(move || {
        block_on(async {
            println!("我等到了{:?}", op_a.await.unwrap());
        })
    });
    let task2 = thread::spawn(move || {
        block_on(async {
            // Ensure we move the producer into this thread but we never resolve
            // it.
            thread::sleep(std::time::Duration::from_millis(10));
            std::mem::drop(op);
            // println!("我发送了了{:?}", op.resolve(String::from("🍓")));
        })
    });
    task2.join().expect("The task2 thread has panicked");
    task1.join().expect("The task1 thread has panicked");
    // assert!(task1.join().is_err());
}

#[allow(unused_must_use)]
#[should_panic(expected = "The task1 thread has panicked")]
#[test]
fn test_promise_out_unresolved_producer_first() {
    let (op, op_a) = Producer::<String>::new();
    let task1 = thread::spawn(move || {
        block_on(async {
            thread::sleep(std::time::Duration::from_millis(10));
            println!("我等到了{:?}", op_a.await.unwrap());
        })
    });
    let task2 = thread::spawn(move || {
        block_on(async {
            // Ensure we move the producer into this thread but we never resolve
            // it.
            std::mem::drop(op);
            // println!("我发送了了{:?}", op.resolve(String::from("🍓")));
        })
    });
    task2.join().expect("The task2 thread has panicked");
    task1.join().expect("The task1 thread has panicked");
    // assert!(task1.join().is_err());
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
            println!("我等到了{:?}", b.await.unwrap());
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
