use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::task::Waker;
use std::{future::Future, task::Poll};
pub mod promise_out;
pub mod pair;

///promise
///
/// # Examples
///
/// ```
/// use promise_out::Producer;
/// let op: Producer<String,String> = Producer::default();
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

pub trait Promise {
    type Output;
    type Error;
    type Waiter : Future;

    fn resolve(self, value: Self::Output);
    fn reject(self, err: Self::Error);
    fn new() -> (Self, Self::Waiter) where Self: Sized;
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
    /// use promise_out::{Promise, Producer};
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
    /// use promise_out::{Promise, Producer};
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
        let producer = Producer::default();
        let consumer = Consumer { promise: producer.promise.clone() };
        (producer, consumer)
    }

}

impl<T, E> Default for Producer<T, E> {
    fn default() -> Self {
        Self {
            promise: Arc::new(Mutex::new(Inner {
                value: None,
                waker: vec![],
            })),
        }
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

#[allow(unused_imports)]
use futures::executor::block_on;
#[allow(unused_imports)]
use std::thread;

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
fn test_promise_resolve_twice() {
    let (a, _b) = Producer::<String, ()>::new();
    a.resolve("hi".into());
    // Not possible. a is consumed.
    // a.resolve("hi".into());
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
