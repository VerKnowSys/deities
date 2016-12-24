/// project name
pub const NAME: &'static str = "Deities";

/// project version from cargo metadata
pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

/// Veles endless loop pause interval
pub static CHECK_INTERVAL: u64 = 1000;

/// Default dir containing services configuration
pub static SERVICES_DIR: &'static str = "/Services";

/// Default glob match for file types we want to process as services configuration
pub static SERVICES_GLOB: &'static str = "*.ini";
