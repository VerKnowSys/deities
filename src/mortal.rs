use std::fmt;
use std::fmt::Display;
use std::io::Error;
use curl::Error as CurlError;
use slack_hook::Error as SlackError;


use service::Service;
use init_fields::InitFields;


#[derive(Debug)]
pub enum Mortal {

    /// Successes:
    OkAllChecks{service: Service, amount: i32},
    OkUrlsChecks{service: Service},
    OkUnixSockCheck{service: Service},
    OkPidAlive{service: Service, pid: i32},
    OkPidInterrupted{service: Service, pid: i32},
    OkPidAlreadyInterrupted{service: Service, pid: i32},
    OkDiskCheck{service: Service},

    /// Failures:
    CheckNoServiceChecks{service: Service},
    CheckPidDead{service: Service, pid: i32},

    RawLoadFailure{file_name: String, cause: Error},
    RawAccessFailure{file_name: String, cause: Error},
    DefinitionDecodeFailure{ini_name: String, cause: Error},
    DefinitionLoadFailure{ini_name: String, cause: Error},

    CheckURL{service: Service, url: String, cause: CurlError},
    CheckURLFail{service: Service, cause: CurlError},
    CheckPidfileMalformed{service: Service},
    CheckPidfileUnaccessible{service: Service, cause: Error},
    CheckUnixSocket{service: Service, cause: Error},
    CheckUnixSocketMissing{service: Service, cause: Error},
    CheckDiskSpace{service: Service},
    CheckDiskInodes{service: Service},

    ServiceNoStartDefined{service: Service},
    ServiceStartFailure{service: Service, cause: Error},

    NotificationFailure{cause: SlackError},

    SanityCheckFailure{message: String},
}


impl Display for Mortal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Mortal: {}", match self {
            &Mortal::OkAllChecks{ref service, ref amount} => format!("Ok: {} successfully passed all: {} checks!", service, amount),
            &Mortal::OkUrlsChecks{ref service} => format!("Ok: {} successfully passed URLs checks!", service),
            &Mortal::OkUnixSockCheck{ref service} => format!("Ok: {} successfully passed UNIX sock checks!", service),
            &Mortal::OkPidAlive{ref service, ref pid} => format!("Ok: Alive pid: {} of service: {}", pid, service),
            &Mortal::OkPidInterrupted{ref service, ref pid} => format!("Ok: Interrupted pid: {} of service: {}", pid, service),
            &Mortal::OkPidAlreadyInterrupted{ref service, ref pid} => format!("Ok: Already interrupted pid: {} of service: {}", pid, service),
            &Mortal::OkDiskCheck{ref service} => format!("Ok: Disk check passed for service: {}", service),

            &Mortal::CheckNoServiceChecks{ref service} => format!("{} has to contain at least single check!", service),
            &Mortal::CheckPidDead{ref service, ref pid} => format!("Found dead pid: {} of {}!", service, pid),

            &Mortal::RawLoadFailure{ref file_name, ref cause} => format!("Can't load a raw file: {}. Reason: {}!", file_name, cause),
            &Mortal::RawAccessFailure{ref file_name, ref cause} => format!("Can't access a raw file: {}. Reason: {}!", file_name, cause),
            &Mortal::DefinitionDecodeFailure{ref ini_name, ref cause} => format!("Failed to decode definition from ini: {}. Reason: {}!", ini_name, cause),
            &Mortal::DefinitionLoadFailure{ref ini_name, ref cause} => format!("Failed to load definition from ini: {}. Reason: {}!", ini_name, cause),

            &Mortal::CheckURL{ref service, ref url, ref cause} => format!("Failed URL check for: {} of: {}. Reason: {}!", url, service, cause),
            &Mortal::CheckURLFail{ref service, ref cause} => format!("Internal CURL failure for: {}. Reason: {}!", service, cause),
            &Mortal::CheckPidfileMalformed{ref service} => format!("Detected malformed pid file of: {}!", service),
            &Mortal::CheckPidfileUnaccessible{ref service, ref cause} => format!("Cannot access pid file for: {}. Reason: {}!", service, cause),
            &Mortal::CheckUnixSocket{ref service, ref cause} => format!("Couldn't connect through UNIX socket: {} of: {}. Reason: {}!", service.unix_socket(), service, cause),
            &Mortal::CheckUnixSocketMissing{ref service, ref cause} => format!("Missing expected UNIX socket: {} of: {}. Reason: {}!", service.unix_socket(), service, cause),
            &Mortal::CheckDiskSpace{ref service} => format!("Disk space check alert! Available: {} MiB!", service.disk_space() / 1024),
            &Mortal::CheckDiskInodes{ref service} => format!("Disk inodes check alert! Available: {} !", service.disk_inodes()),

            &Mortal::ServiceNoStartDefined{ref service} => format!("No 'start' value in configuration of: {}!", service),
            &Mortal::ServiceStartFailure{ref service, ref cause} => format!("Failed to launch commands: {} for {}! Reason: {}", service.clone().start.unwrap_or("#no-commands".to_string()), service, cause),

            &Mortal::NotificationFailure{ref cause} => format!("Failed to send notification! Reason: {}", cause),

            &Mortal::SanityCheckFailure{ref message} => format!("Sanity check failed: {}", message),
        })
    }
}
