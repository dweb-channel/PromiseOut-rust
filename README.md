# promise_out: An Async Promise library for Rust
promiseOut的rust版本

[A promise](http://dist-prog-book.com/chapter/2/futures.html) is a writeable,
single-assignment container, that resolves an associated future. This library
has two principle parts: a promise or producer that accepts a value, and a
future or consumer that can be `.await`ed for that value.

A promise is a convenient way to synchronize value and time.

This library has three variants of promises:

* pair, a single producer, single consumer (spsc) promise;
* poly, a single producer, multiple consumer (spmc) promise;
* and channel, a multiple producer, single consumer (mpsc) promise.

```rust
use promise_out::{Promise, pair::Producer};
use futures::executor::block_on;
use std::thread;

let (promise, consumer) = Producer::<String>::new();

let task1 = thread::spawn(move || block_on(async {
    assert_eq!("Hi", consumer.await.unwrap());
}));
promise.resolve("Hi".into());

task1.join().expect("The task1 thread has panicked.");
```

# Installation

## Edit cargo.toml
```toml
[dependencies]
 promise_out = "1.0.0"
```

OR

## Run command

``` sh
cargo add promise_out
```

# Examples

## Resolve a promise

At its simplest, a promise can be resolved and its result awaited.

```rust
use promise_out::{Promise, pair::Producer};
use futures::executor::block_on;
use std::thread;
let (promise, consumer) = Producer::<String>::new();
let task1 = thread::spawn(move || {
    block_on(async {
        assert_eq!("🍓", consumer.await.unwrap());
    })
});
let task2 = thread::spawn(move || {
    block_on(async {
        promise.resolve(String::from("🍓"));
    })
});
task1.join().expect("The task1 thread has panicked.");
task2.join().expect("The task2 thread has panicked.");
```

## Reject a promise

Typically a promise library will also offer a `reject()` method to serve a user
specified error. However, resolving with an `Err(E)` for a
`Promise<Result<T,E>>` serves the same purpose.

```rust
use promise_out::{Promise, pair::Producer};
use futures::executor::block_on;
use std::thread;
let (promise, consumer) = Producer::<Result<(), &'static str>>::new();
let task1 = thread::spawn(|| {
    block_on(async {
        assert_eq!(Err("reject!"), consumer.await.unwrap());
    })
});
let task2 = thread::spawn(|| {
    block_on(async {
        promise.resolve(Err("reject!"));
    })
});
task1.join().expect("The task1 thread has panicked.");
task2.join().expect("The task2 thread has panicked.");
```

## Cancel a promise

A promise can be cancelled by dropping its producer. In such a case the consumer
will want to take care that they handle the possible error in `Result<T,
promise_out::Error>` from `consumer.await`.

```rust
use promise_out::{Promise, Error, pair::Producer};
use futures::executor::block_on;
use std::thread;
let (promise, consumer) = Producer::<Result<String, &'static str>>::new();
let task1 = thread::spawn(|| {
    block_on(async {
        assert_eq!(Error::ProducerDropped, consumer.await.unwrap_err());
    })
});
let task2 = thread::spawn(|| {
    block_on(async {
        // Dropping a promise is canceling.
        std::mem::drop(promise);
    })
});
task1.join().expect("The task1 thread has panicked.");
task2.join().expect("The task2 thread has panicked.");
```

## Accidentally drop a promise

Losing a promise will not cause the consumer to wait forever[^1]. It will be handled
like a cancellation and return a `Err(promise_out::Error::ProducerDropped)`. In the
below example a panic in `task2` prevents it from resolving the promise. This
then causes a panic in `task1` since it `expect`s an `Ok(_)` result.

```rust
use promise_out::{Promise, Error, pair::Producer};
use futures::executor::block_on;
use std::thread;
let (promise, consumer) = Producer::<String>::new();
let task1 = thread::spawn(|| {
    block_on(async {
        assert_eq!("🍓", consumer.await.expect("Promise Error"));
    })
});
let task2 = thread::spawn(|| {
    block_on(async {
        // ... Simulate a problem.
        panic!("Had a problem.");
        promise.resolve(String::from("🍓"));
    })
});
assert!(task1.join().is_err());
assert!(task2.join().is_err());
```

### Await in multiple places

We can await in multiple places with a poly promise. Do note that a poly
consumer `.await.unwrap()` returns a `Arc<T>` instead of a simple T that a pair
promise returns.

``` rust
use promise_out::{Promise, Error, poly::Producer};
use futures::executor::block_on;
use std::{thread, sync::Arc};

let (promise, consumer) = Producer::<String>::new();
let consumer2 = consumer.clone();
let task1 = thread::spawn(move || {
    block_on(async {
        let value: Arc<String> = consumer.await.unwrap();
        assert_eq!("🍓", *value);
    })
});
let task2 = thread::spawn(move || {
    block_on(async {
        let value: Arc<String> = consumer2.await.unwrap();
        assert_eq!("🍓", *value);
    })
});
let task3 = thread::spawn(move || {
    block_on(async {
        promise.resolve(String::from("🍓"));
    })
});
task1.join().expect("The task1 thread has panicked");
task2.join().expect("The task2 thread has panicked");
task3.join().expect("The task3 thread has panicked");

```

[^1]: promise_out v1.0.0 and earlier will wait forever.
