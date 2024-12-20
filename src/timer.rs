use std::sync::{Arc, Mutex};
use std::task::Waker;
use std::{future::Future, thread, time::Duration};

pub struct TimeFuture {
    state: Arc<Mutex<State>>,
}
pub struct State {
    pub complete: bool,
    pub waker: Option<Waker>,
}

impl TimeFuture {
    pub fn new(duration: Duration) -> Self {
        let state = Arc::new(Mutex::new(State {
            complete: false,
            waker: None,
        }));
        let thread_state = state.clone();
        thread::spawn(move || {
            println!("time future thread is going to sleep");
            thread::sleep(duration);
            thread_state.lock().unwrap().complete = true;
            if let Some(waker) = thread_state.lock().unwrap().waker.take() {
                waker.wake();
            }
        });
        TimeFuture { state }
    }
}

impl Future for TimeFuture {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut state = self.state.lock().unwrap();
        if state.complete {
            std::task::Poll::Ready(())
        } else {
            state.waker = Some(cx.waker().clone());
            std::task::Poll::Pending
        }
    }
}
