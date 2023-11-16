use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::{future::Future, task::{Poll, Waker}};
use crate::{Promise, Error, WakerState};

/// This `poly::Producer` promise can have many consumers. The consumers may be
/// cloned. The consumers return a `Arc<Result<T,E>>`.
///
/// # Examples
///
/// ```
/// use promise_out::{Promise, poly::Producer};
/// use futures::executor::block_on;
/// use std::thread;
/// let (promise, consumer) = Producer::<String>::new();
/// let consumer2 = consumer.clone();
/// let task1 = thread::spawn(move || block_on(async {
///     println!("Received on task 1 {:?}",  consumer.await.unwrap());
/// }));
/// let task2 = thread::spawn(move || block_on(async {
///     println!("Received on task 2 {:?}",  consumer2.await.unwrap());
/// }));
/// promise.resolve("Hi".into());
/// task1.join().expect("The task1 thread has panicked.");
/// task2.join().expect("The task2 thread has panicked.");
/// ```
#[derive(Debug)]
pub struct Producer<T> {
    promise: Arc<Mutex<Inner<T>>>,
}

#[derive(Clone)]
pub struct Consumer<T> {
    promise: Arc<Mutex<Inner<T>>>,
}

#[derive(Debug)]
struct Inner<T> {
    value: Option<Arc<T>>,
    waker: Result<Vec<Waker>, WakerState>, // This was failing the two promise when only one waker
                       // was kept. Even though many docs insist you only need
                       // to wake the last waker. I don't get it.
                       // https://rust-lang.github.io/async-book/02_execution/03_wakeups.html
}


impl<T> Promise<T> for Producer<T> {
    type Waiter = Consumer<T>;
    #[allow(dead_code)]
    ///promiseOut.resolve
    ///
    /// # Examples
    ///
    /// ```
    /// use promise_out::{Promise, poly::Producer};
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
        promise.value = Some(Arc::new(value));
        if let Ok(mut wakers) = std::mem::replace(&mut promise.waker, Err(WakerState::Tainted)) {
            for waker in wakers.drain(..) {
                waker.wake()
            }
        }
    }

    /// promise.new
    ///
    /// This is a slight fib because we're not implementing Clone, and we aren't
    /// doing that because we're not returning Self. We're returning a
    /// Consumer<T, E> which you can wait on.
    fn new() -> (Self, Self::Waiter) {
        let producer = Self {
                            promise: Arc::new(Mutex::new(Inner {
                                value: None,
                                waker: Err(WakerState::Fresh),
                            })),
                        };
        let consumer = Consumer { promise: producer.promise.clone() };
        (producer, consumer)
    }

}

impl<T> Future for Consumer<T> {
    type Output = Result<Arc<T>, Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut promise = self.promise.lock().unwrap();
        match promise.value {
            Some(ref value) => Poll::Ready(Ok(value.clone())),
            None => {
                match &mut promise.waker {
                    Err(WakerState::Tainted) => Poll::Ready(Err(Error::ProducerDropped)),
                    Err(WakerState::Fresh) => {
                        promise.waker = Ok(vec![cx.waker().clone()]);
                        Poll::Pending
                    }
                    Ok(wakers) => {
                        wakers.push(cx.waker().clone());
                        Poll::Pending
                    }
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
use std::sync::Arc;
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
#[test]
fn test_two_promises_out_resolve() {
    let (op, op_a) = Producer::<String>::new();
    let op_b = op_a.clone();
    let task1 = thread::spawn(move || {
        block_on(async {
            println!("我等到了{:?} task1", op_a.await.unwrap());
        })
    });
    let task2 = thread::spawn(move || {
        block_on(async {
            println!("我等到了{:?} task2", op_b.await.unwrap());
        })
    });
    let task3 = thread::spawn(move || {
        block_on(async {
            println!("我发送了了{:?} task3", op.resolve(String::from("🍓")));
        })
    });
    task1.join().expect("The task1 thread has panicked");
    task2.join().expect("The task2 thread has panicked");
    task3.join().expect("The task3 thread has panicked");
}

#[test]
fn test_promise_out_reject() {
    let (a, b) = Producer::<Result<String, String>>::new();
    let task1 = thread::spawn(|| {
        block_on(async {
            let result: Arc<Result<String, String>> = b.await.unwrap().clone();
            println!("我等到了{:?}", result.as_ref());
        })
    });
    let task2 = thread::spawn(|| {
        block_on(async {
            println!("我发送了了{:?}", a.resolve(Err(String::from("reject!!"))));
        })
    });
    task1.join().expect("The task1 thread has panicked");
    task2.join().expect("The task2 thread has panicked");
}

#[allow(unused_must_use)]
#[test]
fn test_promise_resolve_twice() {
    let (a, _b) = Producer::<String>::new();
    a.resolve("hi".into());
    // Not possible. a is consumed. I love rust.
    // a.resolve("hi".into());
}

}
