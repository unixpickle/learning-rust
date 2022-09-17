use futures_core::Stream;
use std::future::Future;
use std::mem::take;
use std::ops::DerefMut;
use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

struct YielderState<T> {
    value: Option<T>,
}

pub struct YielderFuture<T> {
    state: Arc<Mutex<YielderState<T>>>,
}

impl<T> Future for YielderState<T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.deref_mut().state.lock().unwrap().is_some() {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

pub struct Yielder<T> {
    state: Arc<Mutex<YielderState<T>>>,
}

impl<T> YielderState<T> {
    fn put(&mut self, x: T) -> YielderFuture<T> {
        let locked = self.state.lock().unwrap();
        if locked.value.is_some() {
            panic!("cannot yield more than once without awaiting the previous result");
        }
    }
}

pub struct YielderStream<T, Fut: Future> {
    future: Pin<Box<Fut>>,
    future_done: bool,
    state: Arc<Mutex<YielderState<T>>>,
}

impl<T, Fut: Future> Stream for YielderStream<T, Fut> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        {
            // It is possible the Yielder was moved outside of the
            // future and yield was called on it externally.
            let locked = self.state.lock().unwrap();
            if let Some(x) = take(&mut locked.value) {
                return Poll::Ready(Some(x));
            }
        }

        if self.future_done {
            return Poll::Ready(None);
        } else if let Poll::Ready(_) = self.future.as_mut().poll(cx) {
            self.future_done = true;
        }

        let locked = self.state.lock().unwrap();
        if let Some(x) = take(&mut locked.value) {
            Poll::Ready(Some(x))
        } else {
            Poll::Pending
        }
    }
}

pub fn make_stream<T, F, Fut: Future<Output = ()>>(f: F) -> YielderStream<T, Fut>
where
    F: FnMut(Yielder<T>) -> Fut,
{
}
