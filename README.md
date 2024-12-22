# a basic rust async runtime based on uring-io


example :
```rust
fn main() {
    let mut e = runtime::init();
    e.block_on(async {
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
    });
}

```