/// project name
pub const NAME: &'static str = "Deities";

/// project version from cargo metadata
pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

/// Veles endless loop pause interval
pub static CHECK_INTERVAL: u64 = 3000;

/// timeouts for connection, transfer and dns cache for curl
pub static URLCHECK_TIMEOUT: u64 = 10000;

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
pub static SLACK_WEBHOOK_URL: &'static str = "";

/// default channel to post notifications
pub static SLACK_ALERT_CHANNEL: &'static str = "#ops-status";

/// default host to report as a fallback
pub static DEFAULT_HOSTNAME: &'static str = "localhost";

/// default link to remote vks logo file
pub static DEFAULT_VKS_LOGO: &'static str = "http://dmilith.verknowsys.com/vks.png";

/// default name of notifier bot
pub static DEFAULT_NOTIFICATION_NAME: &'static str = "Failure Reporter";

/// default shell to spawn command with
pub static DEFAULT_SHELL: &'static str = "/bin/sh";

/// default PATH for service
pub static DEFAULT_PATH: &'static str = "/bin:/usr/bin:/sbin:/usr/sbin:/usr/local/bin:/usr/local/sbin";

/// default lock file
pub static DEFAULT_LOCK: &'static str = "/.deities.lock";
