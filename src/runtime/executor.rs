use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};

use std::cell::RefCell;
use std::task::Waker;
use std::thread;
use std::vec::Vec;

type Task = std::pin::Pin<Box<dyn Future<Output = ()>>>;
pub struct Executor {
    task_map: RefCell<HashMap<usize, Task>>,
    task_queue: Arc<Mutex<Vec<usize>>>,
    next_id: usize,
}

pub struct MyWaker {
    thread_handle: thread::Thread,
    task_id: usize,
    task_queue: Arc<Mutex<Vec<usize>>>,
}
impl std::task::Wake for MyWaker {
    fn wake(self: std::sync::Arc<Self>) {
        self.task_queue
            .lock()
            .map(|mut q| q.push(self.task_id))
            .unwrap();
        println!("Task {} is woken up", self.task_id);
        self.thread_handle.unpark();
    }
}

impl Executor {
    pub(crate) fn new() -> Self {
        Self {
            task_map: RefCell::new(HashMap::new()),
            task_queue: Arc::new(Mutex::new(Vec::new())),
            next_id: 0,
        }
    }
    pub fn spawn<F>(&mut self, f: F)
    where
        F: Future<Output = ()> + 'static,
    {
        self.task_map.borrow_mut().insert(self.next_id, Box::pin(f));
        self.task_queue
            .lock()
            .map(|mut q| q.push(self.next_id))
            .unwrap();
        self.next_id += 1;
    }

    fn get_waker(&self, task_id: usize) -> Arc<MyWaker> {
        Arc::new(MyWaker {
            thread_handle:thread::current(),
            task_id,
            task_queue: self.task_queue.clone(),
        })
    }

    pub fn block_on<F>(&mut self, f: F)
    where
        F: Future<Output = ()> + 'static,
    {
        self.spawn(f);
        loop {
            while let Some(task_id) = self.task_queue.lock().map(|mut q| q.pop()).unwrap() {
                let mut borrow_mut = self.task_map.borrow_mut();
                let mut task = borrow_mut.remove(&task_id).unwrap();
                let waker: Waker = self.get_waker(task_id).into();
                let mut cx = std::task::Context::from_waker(&waker);
                match task.as_mut().poll(&mut cx) {
                    std::task::Poll::Ready(_) => continue,
                    std::task::Poll::Pending => {
                        borrow_mut.insert(task_id, task);
                    }
                }
            }
            let num = self.task_map.borrow().len();
            if num == 0 {
                println!("All tasks are done");
                break;
            } else {
                println!("{} tasks pending", num);
                std::thread::park();
                println!("executor thread is woken up");
            }
        }
    }
}
