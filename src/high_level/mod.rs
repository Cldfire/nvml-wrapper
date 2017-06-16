#[cfg(target_os = "linux")]
pub mod event_loop;

pub use self::event_loop::{
    Event,
    EventLoop,
    EventLoopProvider,
};
