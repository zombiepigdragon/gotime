#![allow(clippy::missing_safety_doc)] // later
                                      // #![no_std]

pub mod boxed;
pub mod executor;
mod ffi;
pub mod task;

pub use boxed::GoBox;
pub use task::block_on;
