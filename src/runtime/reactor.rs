use io_uring::{squeue::Entry, IoUring};

use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex, OnceLock};
use std::task::{Context, Waker};

static REACTOR: OnceLock<Reactor> = OnceLock::new();

pub struct Reactor {
    wakers: Arc<Mutex<HashMap<usize, Waker>>>,
    data_lens: Arc<Mutex<HashMap<usize, i32>>>,

    ring: Arc<Mutex<IoUring>>,
    next_id: AtomicUsize,
}

impl Reactor {
    pub fn register_waker(&self, id: usize, cx: &Context) {
        let mut wakers = self.wakers.lock().unwrap();
        wakers.insert(id, cx.waker().clone());
    }
    pub fn register(&self, entry: &Entry) -> bool {
        unsafe {
            self.ring
                .lock()
                .unwrap()
                .borrow_mut()
                .submission()
                .push(entry)
                .is_ok()
        }
    }
    pub fn get_id(&self) -> usize {
        self.next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
    pub fn get_size(&self, id: usize) -> i32 {
        *self.data_lens.lock().unwrap().get(&id).unwrap()
    }
}
pub fn reactor() -> &'static Reactor {
    REACTOR.get().expect("Reactor not started")
}

fn uring_io_loop(
    wakers: Arc<Mutex<HashMap<usize, Waker>>>,
    data_lens: Arc<Mutex<HashMap<usize, i32>>>,
) {
    // then, polling completed requests
    let ring = REACTOR.get().unwrap().ring.clone();
    loop {
        match ring.lock().unwrap().borrow().submit() {
            Ok(_) => {}
            Err(e) => {
                println!("Error submitting to io_uring: {:?}", e);
                break;
            }
        }
        while let Some(entry) = ring.lock().unwrap().borrow_mut().completion().next() {
            let task_id = entry.user_data();

            let wakers = wakers.lock().unwrap();
            if let Some(waker) = wakers.get(&(task_id as usize)) {
                data_lens
                    .lock()
                    .unwrap()
                    .insert(task_id as usize, entry.result());
                waker.wake_by_ref()
            } else {
                println!("No waker found for task {}", task_id);
            }
        }
    }
}
pub fn start() {
    let wakers = Arc::new(Mutex::new(HashMap::new()));
    let data_lens = Arc::new(Mutex::new(HashMap::new()));
    let reactor = Reactor {
        wakers: wakers.clone(),

        ring: Arc::new(Mutex::new(
            IoUring::new(32).expect("Failed to create io_uring"),
        )),
        data_lens: data_lens.clone(),
        next_id: AtomicUsize::new(0),
    };

    REACTOR.set(reactor).ok().expect("Reactor already started");
    std::thread::spawn(|| {
        uring_io_loop(wakers, data_lens);
    });
}
