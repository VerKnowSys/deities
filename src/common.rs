/// project name
pub const NAME: &'static str = "Deities";

/// project version from cargo metadata
pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

/// Veles endless loop pause interval
pub static CHECK_INTERVAL: u64 = 2000;

/// timeouts for connection, transfer and dns cache for curl
pub static URLCHECK_TIMEOUT: u64 = 3000;

/// Default dir containing services configuration
pub static SERVICES_DIR: &'static str = "/Services";

/// Default glob match for file types we want to process as services configuration
pub static SERVICES_GLOB: &'static str = "*.ini";

/// default message contents sent via UNIX socket after connection
pub static UNIX_SOCKET_MSG: &'static [u8; 7] = b"version";

/// default logger level env variable
pub static LOG_ENV: &'static str = "LOG";

/// default path to system cacert.pem (used by curl)
pub static CACERT_PEM: &'static str = "/etc/ssl/cert.pem";

/// default path to system cacert.pem (used by curl)
pub static SLACK_WEBHOOK_URL: &'static str = "https://hooks.slack.com/services/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";

/// default channel to post notifications
pub static SLACK_ALERT_CHANNEL: &'static str = "#dev-status";


