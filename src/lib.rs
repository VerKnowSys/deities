#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate lazy_static;


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
