#[macro_use]
extern crate serde_derive;
// extern crate toml;

// #[macro_use]
// extern crate log;
// extern crate colored;
// extern crate fern;
// extern crate libc;
// extern crate glob;
// extern crate time;
// extern crate uuid;
// extern crate curl;
// extern crate slack_hook;
// extern crate chrono;
// extern crate hostname;
// extern crate uname;
// extern crate users;
// extern crate fs2;

#[macro_use]
extern crate lazy_static;
// extern crate regex;

pub mod common;
pub mod init_fields;
pub mod mortal;
pub mod perun;
pub mod service;
pub mod svarog;
pub mod veles;

pub use common::*;
pub use init_fields::InitFields;
pub use perun::Perun;
pub use service::Service;
pub use svarog::Svarog;
pub use tracing::{debug, error, info, instrument, trace, warn};
pub use veles::Veles;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
