use crate::uring_future;
use std::rc::Rc;
use uring_future::{BufResult, UringRead};
pub struct File {
    pub fd: Rc<std::fs::File>,
}

impl File {
    pub fn open(path: &str) -> Self {
        let file = std::fs::File::open(path).unwrap();
        Self { fd: Rc::new(file) }
    }
    pub async fn read_at(&self, buf: Vec<u8>, offset: u64) -> BufResult {
        let fut = UringRead::new(self.fd.clone(), buf, offset);
        fut.await
        //fut.await;
    }
    pub async fn write_at(&self, buf: Vec<u8>, offset: u64) -> BufResult {
        let fut = UringRead::new(self.fd.clone(), buf, offset);
        fut.await
        //fut.await;
    }
}
