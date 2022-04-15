// common defaults:

/// project name
pub const NAME: &str = "Deities";

/// project version from cargo metadata
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default dir containing services configuration
pub static SERVICES_DIR: &str = "/Services";

/// Default glob match for file types we want to process as services configuration
pub static SERVICES_GLOB: &str = "*.ini";

/// default message contents sent via UNIX socket after connection
pub static UNIX_SOCKET_MSG: &[u8; 7] = b"version";

/// default logger level env variable
pub static LOG_ENV: &str = "LOG";

/// default path to system cacert.pem (used by curl)
pub static CACERT_PEM: &str = "/etc/ssl/cert.pem";

/// default host to report as a fallback
pub static DEFAULT_HOSTNAME: &str = "localhost";

/// default link to remote vks logo file
pub static DEFAULT_VKS_LOGO: &str = "http://dmilith.verknowsys.com/vks.png";

/// default name of notifier bot
pub static DEFAULT_NOTIFICATION_NAME: &str = "Failure Reporter";

/// default shell to spawn command with
pub static DEFAULT_SHELL: &str = "/bin/sh";

/// default PATH for service
pub static DEFAULT_PATH: &str = "/bin:/usr/bin:/sbin:/usr/sbin:/usr/local/bin:\
                                         /usr/local/sbin";

/// default lock file
pub static DEFAULT_LOCK: &str = "/.deities.lock";

// Override-able defaults (fallbacks if value unset in ini file):

/// Veles endless loop pause interval
pub static CHECKS_INTERVAL: u64 = 3000;

/// timeouts for connection, transfer and dns cache for curl
pub static CHECKS_URL_TIMEOUT: u64 = 10000;

/// pause after each signal sent by death_watch to get rid of live pid
pub static DEATHWATCHES_INTERVAL: u64 = 2000;

/// minimum disk space required for disk
pub static DISK_MINIMUM_SPACE: i64 = 3000; // in MiB

/// minimum disk inodes on disk
pub static DISK_MINIMUM_INODES: i64 = 4096;

/// default channel to post notifications
pub static SLACK_ALERT_CHANNEL: &str = "#ops-status";
