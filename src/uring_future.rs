use io_uring::{opcode, types};
use std::future::Future;
use std::os::fd::AsRawFd;
use std::rc::Rc;
use std::task::Poll;

use crate::runtime;
use pin_project::pin_project;
pub type BufResult = (std::io::Result<i32>, Vec<u8>);

#[pin_project]
pub struct UringRead {
    submited: bool,
    buf: Vec<u8>,
    offset: u64,
    std_file: Rc<std::fs::File>,
    id: usize,
}

impl UringRead {
    pub fn new(std_file: Rc<std::fs::File>, buf: Vec<u8>, offset: u64) -> Self {
        let id = runtime::reactor().get_id();
        Self {
            submited: false,
            buf,
            offset,
            std_file,
            id,
        }
    }
}
impl Future for UringRead {
    type Output = BufResult;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if !self.submited {
            let this = self.as_mut().project();
            let buf_ptr: &mut [u8] = this.buf.as_mut();
            let ptr = buf_ptr.as_mut_ptr();
            let len: u32 = buf_ptr.len() as _;
            let read_op = opcode::Read::new(types::Fd(this.std_file.as_raw_fd()), ptr, len)
                .offset(*this.offset);
            let entry = read_op.build().user_data(*this.id as _);
            if !runtime::reactor().register(&entry) {
                return Poll::Pending;
            }
            *this.submited = true;
            runtime::reactor().register_waker(*this.id, cx);
            Poll::Pending
        } else {
            let buf_size = runtime::reactor().get_size(self.id );
            Poll::Ready((Ok(buf_size), self.buf.clone()))
        }
    }
}
