# PromiseOut-rust
promiseOut的rust版本


```rust
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
```
