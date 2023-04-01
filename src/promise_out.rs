use std::sync::{
    mpsc::{channel, Receiver, RecvError, Sender},
    Arc, Mutex,
};
/// promiseOut channel version
#[derive(Debug)]
pub struct PromiseOutChannel<T> {
    sender: Sender<T>,
    receiver: Arc<Mutex<Receiver<T>>>,
}

impl<T> Clone for PromiseOutChannel<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
        }
    }
}

impl<T> Default for PromiseOutChannel<T> {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            sender: tx,
            receiver: Arc::new(Mutex::new(rx)),
        }
    }
}

impl<T> PromiseOutChannel<T> {
    pub fn resolve(self, value: T) {
        self.sender.send(value).unwrap();
    }

    pub fn reject(self, err: T) {
        self.sender.send(err).unwrap();
    }

    pub fn await_promise(&self) -> Result<T, RecvError> {
        self.receiver.lock().unwrap().recv()
    }
}
