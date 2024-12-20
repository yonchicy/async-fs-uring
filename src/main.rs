mod fs;
mod runtime;
mod timer;

use std::time::Duration;


fn main() {
    let mut e = runtime::init();
    e.block_on(timer::TimeFuture::new(Duration::from_secs(2)));
}
