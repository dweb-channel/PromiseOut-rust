use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::task::Waker;
use std::{future::Future, task::Poll};
pub mod promise_out;

///promiseOut
///
/// # Examples
///
/// ```
/// let op: PromiseOut<String> = PromiseOut::default();
/// ```
#[derive(Clone, Debug)]
pub struct PromiseOut<T> {
    promise: Arc<Mutex<Promise<T>>>,
}
#[derive(Clone, Debug)]
pub struct Promise<T> {
    completed: bool,
    value: Arc<Result<T, String>>,
    waker: Option<Waker>,
}

impl<T> PromiseOut<T>
where
    T: Clone,
{
    #[allow(dead_code)]
    ///promiseOut.resolve
    ///
    /// # Examples
    ///
    /// ```
    /// let op: PromiseOut<String> = PromiseOut::default();
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
        promise.completed = true;
        promise.value = Arc::new(Ok(value));
        if let Some(waker) = promise.waker.take() {
            waker.wake()
        }
    }
    ///promiseOut.reject
    ///
    /// # Examples
    ///
    /// ```
    /// let op: PromiseOut<String> = PromiseOut::default();
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
    pub fn reject(self, err: String) {
        let mut promise = self.promise.lock().unwrap();
        promise.completed = true;
        promise.value = Arc::new(Err(err));
        if let Some(waker) = promise.waker.take() {
            waker.wake()
        }
    }
}

impl<T> Default for PromiseOut<T> {
    fn default() -> Self {
        Self {
            promise: Arc::new(Mutex::new(Promise {
                completed: false,
                value: Arc::new(Err(String::from("init"))),
                waker: None,
            })),
        }
    }
}

impl<T> Future for PromiseOut<T>
where
    T: Debug + Clone + 'static,
{
    type Output = Arc<Result<T, String>>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut promsie = self.promise.lock().unwrap();
        if promsie.completed {
            let value = promsie.value.clone();
            // let value = Ok(value);
            Poll::Ready(value)
        } else {
            promsie.waker.replace(cx.waker().clone());
            Poll::Pending
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
    let op: PromiseOut<String> = PromiseOut::default();
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

#[test]
fn test_promise_out_reject() {
    let op: PromiseOut<String> = PromiseOut::default();
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
