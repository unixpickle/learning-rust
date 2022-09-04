use std::future::Future;
use std::mem::take;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread::{sleep, spawn};
use std::time::Duration;
use tokio::runtime::Runtime;

fn main() {
    run_with_tokio(async {
        async_sleep(Duration::from_secs(1)).await;
        println!("yo 1");
        async_sleep(Duration::from_secs(1)).await;
        println!("yo 2");
    });
}

fn run_with_tokio<F: Future>(f: F) -> F::Output {
    let rt = Runtime::new().unwrap();
    rt.block_on(f)
}

#[derive(Default)]
enum SleepFutureResult {
    #[default]
    Unstarted,
    Waiting(Waker),
    Done,
}

struct SleepFuture {
    delay: Duration,
    result: Arc<Mutex<SleepFutureResult>>,
}

impl<'a> Future for SleepFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let lock_copy = self.result.clone();
        let delay_copy = self.delay.clone();
        let result: &mut SleepFutureResult = &mut self.result.lock().unwrap();
        let old_result = take(result);
        let (new_result, return_val) = match old_result {
            SleepFutureResult::Unstarted => {
                spawn(move || {
                    sleep(delay_copy);
                    let locked: &mut SleepFutureResult = &mut lock_copy.lock().unwrap();
                    let old_value = take(locked);
                    *locked = SleepFutureResult::Done;
                    if let SleepFutureResult::Waiting(w) = old_value {
                        w.wake();
                    };
                });
                (
                    SleepFutureResult::Waiting(cx.waker().clone()),
                    Poll::Pending,
                )
            }
            SleepFutureResult::Waiting(_) => (
                SleepFutureResult::Waiting(cx.waker().clone()),
                Poll::Pending,
            ),
            SleepFutureResult::Done => (SleepFutureResult::Done, Poll::Ready(())),
        };
        *result = new_result;
        return_val
    }
}

fn async_sleep(duration: Duration) -> SleepFuture {
    SleepFuture {
        delay: duration,
        result: Arc::new(Mutex::new(SleepFutureResult::Unstarted)),
    }
}
