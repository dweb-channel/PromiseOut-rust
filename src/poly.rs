use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::{future::Future, task::{Poll, Waker}};
use crate::Promise;

/// This `poly::Producer` promise can have many consumers. The consumers may be
/// cloned. The consumers return a `Arc<Result<T,E>>`.
///
/// # Examples
///
/// ```
/// use promise_out::{Promise, poly::Producer};
/// use futures::executor::block_on;
/// use std::thread;
/// let (promise, consumer) = Producer::<String, String>::new();
/// let consumer2 = consumer.clone();
/// let task1 = thread::spawn(move || block_on(async {
///     println!("Received on task 1 {:?}",  consumer.await);
/// }));
/// let task2 = thread::spawn(move || block_on(async {
///     println!("Received on task 2 {:?}",  consumer2.await);
/// }));
/// promise.resolve("Hi".into());
/// task1.join().expect("The task1 thread has panicked.");
/// task2.join().expect("The task2 thread has panicked.");
/// ```
#[derive(Debug)]
pub struct Producer<T, E> {
    promise: Arc<Mutex<Inner<T, E>>>,
}

#[derive(Clone)]
pub struct Consumer<T, E> {
    promise: Arc<Mutex<Inner<T, E>>>,
}

#[derive(Debug)]
struct Inner<T, E> {
    value: Option<Arc<Result<T, E>>>,
    waker: Vec<Waker>, // This was failing the two promise when only one waker
                       // was kept. Even though many docs insist you only need
                       // to wake the last waker. I don't get it.
                       // https://rust-lang.github.io/async-book/02_execution/03_wakeups.html
}


impl<T, E> Promise for Producer<T,E> {
    type Output = T;
    type Error = E;
    type Waiter = Consumer<T,E>;
    #[allow(dead_code)]
    ///promiseOut.resolve
    ///
    /// # Examples
    ///
    /// ```
    /// use promise_out::{Promise, poly::Producer};
    /// use futures::executor::block_on;
    /// use std::thread;
    /// let (op, op_a) = Producer::<String, ()>::new();
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
        promise.value = Some(Arc::new(Ok(value)));
        for waker in promise.waker.drain(..) {
            waker.wake()
        }
    }
    ///promiseOut.reject
    ///
    /// # Examples
    ///
    /// ```
    /// use promise_out::{Promise, poly::Producer};
    /// use futures::executor::block_on;
    /// use std::thread;
    /// let (op, op_a) = Producer::<(), String>::new();
    /// let task1 = thread::spawn(move || block_on(async {
    ///     println!("我等到了{:?}",  op_a.await);
    /// }));
    /// let task2 = thread::spawn(move || block_on(async {
    ///     println!("我发送了{:?}", op.reject(String::from("💥")));
    /// }));
    /// task1.join().expect("The task1 thread has panicked");
    /// task2.join().expect("The task2 thread has panicked");
    /// ```
    #[allow(dead_code)]
    fn reject(self, err: E) {
        let mut promise = self.promise.lock().unwrap();
        promise.value = Some(Arc::new(Err(err)));
        for waker in promise.waker.drain(..) {
            waker.wake()
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
                                waker: vec![],
                            })),
                        };
        let consumer = Consumer { promise: producer.promise.clone() };
        (producer, consumer)
    }

}

impl<T, E> Future for Consumer<T, E> {
    type Output = Arc<Result<T, E>>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut promise = self.promise.lock().unwrap();
        match promise.value {
            Some(ref value) => Poll::Ready(value.clone()),
            None => {
                promise.waker.push(cx.waker().clone());
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
    let (op, op_a) = Producer::<String, ()>::new();
    let task1 = thread::spawn(move || {
        block_on(async {
            println!("我等到了{:?}", op_a.await);
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
    let (op, op_a) = Producer::<String, ()>::new();
    let op_b = op_a.clone();
    let task1 = thread::spawn(move || {
        block_on(async {
            println!("我等到了{:?} task1", op_a.await);
        })
    });
    let task2 = thread::spawn(move || {
        block_on(async {
            println!("我等到了{:?} task2", op_b.await);
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
    let (a, b) = Producer::<String, String>::new();
    let task1 = thread::spawn(|| {
        block_on(async {
            println!("我等到了{:?}", b.await);
        })
    });
    let task2 = thread::spawn(|| {
        block_on(async {
            println!("我发送了了{:?}", a.reject(String::from("reject!!")));
        })
    });
    task1.join().expect("The task1 thread has panicked");
    task2.join().expect("The task2 thread has panicked");
}

#[allow(unused_must_use)]
#[test]
fn test_promise_resolve_twice() {
    let (a, _b) = Producer::<String, ()>::new();
    a.resolve("hi".into());
    // Not possible. a is consumed.
    // a.resolve("hi".into());
}

}
