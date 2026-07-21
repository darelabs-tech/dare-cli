//! Safe filesystem operations under [`ProjectRoot`].

mod atomic;
mod backup;
mod lock;

pub use atomic::{atomic_write, read_to_string};
pub use backup::{backup, restore};
pub use lock::FileLock;
