mod executor;
mod reactor;

pub use executor::Executor;

pub fn init() -> Executor {
    Executor::new()
}
