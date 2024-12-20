mod fs;
mod runtime;
mod timer;
mod uring_future;

use std::time::Duration;

fn main() {
    let mut e = runtime::init();
    e.spawn(timer::TimeFuture::new(Duration::from_secs(2)));
    e.block_on(async_func());
}


async fn async_func(){
    //timer::TimeFuture::new(Duration::from_secs(2)).await;
    let file = fs::File::open("Cargo.toml");
    let buf = vec![0; 1_024];
    let res = file.read_at(buf,0).await;
    match res {
        (Ok(n), buf) => {
            println!("Read {} bytes", n);
            println!("First 10 bytes: {}", std::string::String::from_utf8_lossy(&buf[..n as usize]));
        }
        _ => {
            println!("Error reading file");
        }
        
    }


}
