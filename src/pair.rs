use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::task::Waker;
use std::{future::Future, task::Poll};

///promiseOut
///
/// # Examples
///
/// ```
/// use promise_out::pair::Promise;
/// let (producer, consumer) = Promise::<String,String>::pair();
/// ```
#[derive(Debug)]
pub struct Promise<T, E> {
    promise: Arc<Mutex<Inner<T, E>>>,
}

pub struct Consumer<T, E> {
    promise: Arc<Mutex<Inner<T, E>>>,
}

#[derive(Debug)]
struct Inner<T, E> {
    value: Option<Result<T, E>>,
    waker: Option<Waker>,
}

impl<T, E> Promise<T, E> {
    #[allow(dead_code)]
    ///promiseOut.resolve
    ///
    /// # Examples
    ///
    /// ```
    /// use promise_out::pair::Promise;
    /// use futures::executor::block_on;
    /// use std::thread;
    /// let (op, op_a) = Promise::<String, String>::pair();
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
    /// use promise_out::pair::Promise;
    /// use futures::executor::block_on;
    /// use std::thread;
    /// let (op, op_a) = Promise::<String, String>::pair();
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
        promise.value = Some(Err(err));
        if let Some(waker) = promise.waker.take() {
            waker.wake()
        }
    }
}

impl<T, E> Promise<T, E> {
    pub fn pair() -> (Self, Consumer<T,E>) {
        let inner = Arc::new(Mutex::new(Inner {
                value: None,
                waker: None,
            }));
        (Self { promise: inner.clone() }, Consumer { promise: inner })
    }
}

impl<T, E> Future for Consumer<T, E> {
    type Output = Result<T, E>;

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

#[allow(unused_imports)]
use futures::executor::block_on;
#[allow(unused_imports)]
use std::thread;

#[allow(unused_must_use)]
#[test]
fn test_promise_out_resolve() {
    let (op, op_a) = Promise::<String, String>::pair();
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

#[test]
fn test_promise_out_reject() {
    let (a, b) = Promise::<String, String>::pair();
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
