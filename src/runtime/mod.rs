mod executor;
mod reactor;

pub use executor::Executor;

pub fn init() -> Executor {
    reactor::start();
    Executor::new()
}
pub use reactor::reactor;
