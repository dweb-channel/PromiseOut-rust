use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::task::Waker;
use std::{future::Future, task::Poll};
pub mod promise_out;
pub mod pair;

///promiseOut
///
/// # Examples
///
/// ```
/// use promise_out::Promise;
/// let op: Promise<String,String> = Promise::default();
/// ```
#[derive(Clone, Debug)]
pub struct Promise<T, E> {
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

impl<T, E> Promise<T, E> {
    #[allow(dead_code)]
    ///promiseOut.resolve
    ///
    /// # Examples
    ///
    /// ```
    /// use promise_out::Promise;
    /// use futures::executor::block_on;
    /// use std::thread;
    /// let op: Promise<String, String> = Promise::default();
    /// let op_a  = op.clone();
    /// let task1 = thread::spawn(move || block_on(async {
    ///     println!("我等到了{:?}",  op_a.await);
    /// }));
    /// let task2 = thread::spawn(move || block_on(async {
    ///     println!("我发送了{:?}", op.resolve(String::from("🍓")));
    /// }));
    /// task1.join().expect("The task1 thread has panicked");
    /// task2.join().expect("The task2 thread has panicked");
    /// ```
    pub fn resolve(self, value: T) {
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
    /// use promise_out::Promise;
    /// use futures::executor::block_on;
    /// use std::thread;
    /// let op: Promise<String, String> = Promise::default();
    /// let op_a  = op.clone();
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
    pub fn reject(self, err: E) {
        let mut promise = self.promise.lock().unwrap();
        promise.value = Some(Arc::new(Err(err)));
        for waker in promise.waker.drain(..) {
            waker.wake()
        }
    }
}

impl<T, E> Default for Promise<T, E> {
    fn default() -> Self {
        Self {
            promise: Arc::new(Mutex::new(Inner {
                value: None,
                waker: vec![],
            })),
        }
    }
}

impl<T, E> Future for Promise<T, E> {
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
    let op: Promise<String, String> = Promise::default();
    let op_a = op.clone();
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
    let op: Promise<String, String> = Promise::default();
    let op_a = op.clone();
    let op_b = op.clone();
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
    let op: Promise<String, String> = Promise::default();
    let a = op.clone();
    let task1 = thread::spawn(|| {
        block_on(async {
            println!("我等到了{:?}", a.await);
        })
    });
    let b = op.clone();
    let task2 = thread::spawn(|| {
        block_on(async {
            println!("我发送了了{:?}", b.reject(String::from("reject!!")));
        })
    });
    task1.join().expect("The task1 thread has panicked");
    task2.join().expect("The task2 thread has panicked");
}
