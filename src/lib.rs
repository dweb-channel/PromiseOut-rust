use async_std::task::{block_on, Waker};
use async_std::{future::Future, io::Error, task::Poll};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone)]
pub(crate) struct PromiseOut<'a,T> {
    promise: Arc<Mutex<Promise<'a , T>>>,
}

pub(crate) struct Promise<'a,T> {
    completed: bool,
    value: Arc<Option<Result<&'a T, Error>>>,
    waker: Option<Waker>,
}

impl<T> Clone for Promise<'_, T> {
    fn clone(&self) -> Self {
        Self { completed: self.completed.clone(), value: self.value.clone(), waker: self.waker.clone() }
    }
}

impl<T> PromiseOut<'_,  T> {
    pub fn resolve(self, value: T) {
        let mut promise = self.promise.lock().unwrap();
        promise.completed = true;
        promise.value = Arc::new(Some(Ok(&value)));
    }
    pub fn reject(self, err: Error) {
        let mut promise = self.promise.lock().unwrap();
        promise.completed = true;
        promise.value = Arc::new(Some(Err(err)));
    }
}

impl<T> Default for PromiseOut<'_, T> {
    fn default() -> Self {
        Self {
            promise: Arc::new(Mutex::new(Promise {
                completed: false,
                value: Arc::new(None),
                waker: None,
            })),
        }
    }
}

impl< T> Future for PromiseOut<'static,  T> {
    type Output =&'static Result<&'static T, Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut promsie = self.promise.lock().unwrap();
        if promsie.completed {
            let value =promsie.value.as_ref().unwrap();
            // let value = Ok(value);
            Poll::Ready(value)
        } else {
            promsie.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

#[test]
fn test_promiseOut() {
    let op: PromiseOut<String> = PromiseOut::default();
    thread::spawn(move || {
        async {
            println!("我等到了{:?}", &op.await);
        }
    });

    thread::spawn( || {
        async {
            println!("我发送了了{:?}", op.resolve(String::from("🍓")));
        }
    });

}
