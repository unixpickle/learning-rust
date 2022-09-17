use futures_core::Stream;
use std::future::Future;
use std::mem::take;
use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

struct YielderState<T: Unpin> {
    value: Option<T>,
}

pub struct YielderFuture<T: Unpin> {
    state: Arc<Mutex<YielderState<T>>>,
}

impl<T: Unpin> Future for YielderFuture<T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        if self.state.lock().unwrap().value.is_some() {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

pub struct Yielder<T: Unpin> {
    state: Arc<Mutex<YielderState<T>>>,
}

impl<T: Unpin> Yielder<T> {
    fn put(&mut self, x: T) -> YielderFuture<T> {
        let mut locked = self.state.lock().unwrap();
        if locked.value.is_some() {
            panic!("cannot yield more than once without awaiting the previous result");
        }
        locked.value = Some(x);
        YielderFuture {
            state: self.state.clone(),
        }
    }
}

pub struct YielderStream<T: Unpin, Fut: Future> {
    future: Pin<Box<Fut>>,
    future_done: bool,
    state: Arc<Mutex<YielderState<T>>>,
}

impl<T: Unpin, Fut: Future> Stream for YielderStream<T, Fut> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        {
            // It is possible the Yielder was moved outside of the
            // future and yield was called on it externally.
            let mut locked = self.state.lock().unwrap();
            if let Some(x) = take(&mut locked.value) {
                return Poll::Ready(Some(x));
            }
        }

        if self.future_done {
            return Poll::Ready(None);
        } else if let Poll::Ready(_) = self.future.as_mut().poll(cx) {
            self.future_done = true;
        }

        let mut locked = self.state.lock().unwrap();
        if let Some(x) = take(&mut locked.value) {
            Poll::Ready(Some(x))
        } else {
            if self.future_done {
                Poll::Ready(None)
            } else {
                Poll::Pending
            }
        }
    }
}

pub fn make_stream<T: Unpin, F, Fut: Future<Output = ()>>(f: F) -> YielderStream<T, Fut>
where
    F: FnOnce(Yielder<T>) -> Fut,
{
    let state = Arc::new(Mutex::new(YielderState { value: None }));
    let yielder = Yielder {
        state: state.clone(),
    };
    YielderStream {
        future: Box::pin(f(yielder)),
        future_done: false,
        state: state,
    }
}
