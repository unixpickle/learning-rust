mod generators;

use crate::generators::make_stream;
use futures_util::StreamExt;
use std::future::Future;
use std::mem::take;
use std::pin::Pin;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::thread::{sleep, spawn};
use std::time::Duration;
use tokio::runtime::Runtime;

fn main() {
    println!("running with tokio");
    run_with_tokio(my_function());

    println!("running with simple runner");
    run_with_simple_runner(my_function());
}

async fn my_function() {
    async_sleep(Duration::from_secs(1)).await;
    println!("yo 1");
    async_sleep(Duration::from_secs(1)).await;
    println!("yo 2");

    let gen = make_stream::<i32, _, _>(|mut yielder| async move {
        for i in 0..10 {
            async_sleep(Duration::from_millis(109)).await;
            yielder.put(i).await;
        }
    });
    let collected: Vec<i32> = gen.collect().await;
    println!(
        "{}",
        collected
            .into_iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<String>>()
            .join(", ")
    );
}

fn run_with_tokio<F: Future>(f: F) -> F::Output {
    let rt = Runtime::new().unwrap();
    rt.block_on(f)
}

fn run_with_simple_runner<F: Future>(f: F) -> F::Output {
    let (tx, rx) = channel();
    let mut pinned = Box::pin(f);
    let waker = channel_waker(tx);
    let mut ctx = Context::from_waker(&waker);
    let res = loop {
        let res = pinned.as_mut().poll(&mut ctx);
        if let Poll::Ready(x) = res {
            break x;
        }
        rx.recv().unwrap();
    };
    drop(ctx);
    drop(waker);
    if let Ok(_) = rx.recv() {
        panic!("not all wakers released");
    }
    res
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

fn channel_waker(ch: Sender<()>) -> Waker {
    unsafe { Waker::from_raw(raw_channel_waker(ch)) }
}

unsafe fn raw_channel_waker(ch: Sender<()>) -> RawWaker {
    let x = Box::into_raw(Box::new(ch));
    RawWaker::new(
        x as *const (),
        &RawWakerVTable::new(
            |ch_ptr| -> RawWaker {
                raw_channel_waker((ch_ptr as *const Sender<()>).as_ref().unwrap().clone())
            },
            |ch_ptr| {
                Box::from_raw(ch_ptr as *mut Sender<()>).send(()).unwrap();
            },
            |ch_ptr| {
                (ch_ptr as *const Sender<()>)
                    .as_ref()
                    .unwrap()
                    .send(())
                    .unwrap();
            },
            |ch_ptr| drop(Box::from_raw(ch_ptr as *mut Sender<()>)),
        ),
    )
}
