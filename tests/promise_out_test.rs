#[cfg(test)]
mod tests {
    use promise_out::promise_out::PromiseOutChannel;
    use std::{thread, time::Duration};

    #[test]
    fn test_promise_out() {
        let promise = PromiseOutChannel::<i32>::default();
        let promise_clone = promise.clone();

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(1000));
            promise_clone.resolve(42);
        });

        let result = promise.await_promise().unwrap();
        assert_eq!(result, 42);
    }
}
