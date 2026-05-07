#![forbid(unsafe_code)]

pub mod stream;
pub mod types;

pub use stream::EventStream;
pub use types::{DriveEvent, EventAction};
