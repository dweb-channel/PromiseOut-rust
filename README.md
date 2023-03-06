# PromiseOut-rust
promiseOut的rust版本

## add cargo.toml
```toml
[dependencies]
 promise_out = "0.1.0"
```

## example
```rust
#[test]
fn test_promise_out_resolve() {
    let op: PromiseOut<String> = PromiseOut::default();
    let op_a  = op.clone();
    let task1 = thread::spawn(move || block_on(async {
        println!("我等到了{:?}",  op_a.await);
    }));
    let task2 = thread::spawn(move || block_on(async {
        println!("我发送了了{:?}", op.resolve(String::from("🍓")));
    }));
    task1.join().expect("The task1 thread has panicked");
    task2.join().expect("The task2 thread has panicked");
}

#[test]
fn test_promise_out_reject() {
    let op: PromiseOut<String> = PromiseOut::default();
    let a = op.clone();
    let task1 = thread::spawn(|| block_on(async {
        println!("我等到了{:?}", a.await);
    }));
    let b = op.clone();
    let task2 = thread::spawn(|| block_on(async {
        println!("我发送了了{:?}", b.reject(String::from("reject!!")));
    }));
    task1.join().expect("The task1 thread has panicked");
    task2.join().expect("The task2 thread has panicked");
}
```
