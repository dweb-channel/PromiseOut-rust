#[allow(unused_imports)]
use std::{
    sync::mpsc::{channel, Receiver, RecvError, Sender},
    thread,
};

/// promiseOut channel version
#[derive(Debug)]
pub struct PromiseOut<T> {
    pub sender: Sender<T>,
    pub receiver: Receiver<T>,
}

impl <T> Clone for PromiseOut<T> {
    fn clone(&self) -> Self {
        Self { sender: self.sender.clone(), receiver: self.receiver.clone() }
    }
}

impl<T> Default for PromiseOut<T> {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            sender: tx,
            receiver: rx,
        }
    }
}

impl<T> PromiseOut<T> {
    #[allow(dead_code)]
    pub fn resolve(self, value: T) {
        self.sender.send(value).unwrap();
    }
    #[allow(dead_code)]
    pub fn reject(self, err: T) {
        self.sender.send(err).unwrap();
    }
    #[allow(dead_code)]
    pub fn await_promise(self) -> Result<T, RecvError> {
        self.receiver.recv()
    }
}

#[test]
fn test() {
    let promsieOut: PromiseOut<String> = PromiseOut::default();
    let promsieOut1 = promsieOut.clone();
    let sender = thread::spawn( || promsieOut1.resolve("Hello, thread".to_owned()));

    let receiver = thread::spawn( || {
        let value = promsieOut.await_promise();
        println!("value ==>{:?}", value);
    });

    sender.join().expect("The sender thread has panicked");
    receiver.join().expect("The receiver thread has panicked");
}
