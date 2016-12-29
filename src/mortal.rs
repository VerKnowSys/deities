use service::Service;
use std::fmt;
use std::fmt::Display;

use std::io::Error;
use curl::Error as CurlError;
use slack_hook::Error as SlackError;
use std::num::ParseIntError;


#[derive(Debug)]
pub enum Mortal {

    CheckPassed,
    CheckNameEmpty{service: Service},
    CheckNoServiceChecks{service: Service},
    CheckPidDead{service: Service, pid: i32},

    RawLoadFailure{file_name: String, cause: Error},
    RawAccessFailure{file_name: String, cause: Error},
    DefinitionDecodeFailure{ini_name: String, cause: Error},
    DefinitionLoadFailure{ini_name: String, cause: Error},

    CheckURL{service: Service, url: String, cause: CurlError},
    CheckURLFail{service: Service, cause: CurlError},
    CheckPidfileMalformed{service: Service, cause: ParseIntError},
    CheckPidfileUnaccessible{service: Service, cause: Error},
    CheckUnixSocket{service: Service, cause: Error},
    CheckUnixSocketMissing{service: Service, cause: Error},

    ServiceNoStartDefined{service: Service},
    ServiceStartFailure{service: Service, cause: Error},

    NotificationFailure{cause: SlackError},
}


impl Display for Mortal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Mortal: {}", match self {
            &Mortal::CheckPassed => "No error?".to_string(),
            &Mortal::CheckNameEmpty{ref service} => format!("{} has unset 'name' value in configuration file: {}!", service, service.ini_file()),
            &Mortal::CheckNoServiceChecks{ref service} => format!("{} has to contain at least single check!", service),
            &Mortal::CheckPidDead{ref service, ref pid} => format!("Found dead pid: {} of {}!", service, pid),

            &Mortal::RawLoadFailure{ref file_name, ref cause} => format!("Can't load a raw file: {}. Reason: {}!", file_name, cause),
            &Mortal::RawAccessFailure{ref file_name, ref cause} => format!("Can't access a raw file: {}. Reason: {}!", file_name, cause),
            &Mortal::DefinitionDecodeFailure{ref ini_name, ref cause} => format!("Failed to decode definition from ini: {}. Reason: {}!", ini_name, cause),
            &Mortal::DefinitionLoadFailure{ref ini_name, ref cause} => format!("Failed to load definition from ini: {}. Reason: {}!", ini_name, cause),

            &Mortal::CheckURL{ref service, ref url, ref cause} => format!("Failed URL check for: {} of: {}. Reason: {}!", url, service, cause),
            &Mortal::CheckURLFail{ref service, ref cause} => format!("Internal CURL failure for: {}. Reason: {}!", service, cause),
            &Mortal::CheckPidfileMalformed{ref service, ref cause} => format!("Detected malformed pid file of: {}. Reason: {}!", service, cause),
            &Mortal::CheckPidfileUnaccessible{ref service, ref cause} => format!("Cannot access pid file for: {}. Reason: {}!", service, cause),
            &Mortal::CheckUnixSocket{ref service, ref cause} => format!("Couldn't connect through UNIX socket: {} of: {}. Reason: {}!", service.unix_socket(), service, cause),
            &Mortal::CheckUnixSocketMissing{ref service, ref cause} => format!("Missing expected UNIX socket: {} of: {}. Reason: {}!", service.unix_socket(), service, cause),

            &Mortal::ServiceNoStartDefined{ref service} => format!("No 'start' value in configuration of: {}!", service),
            &Mortal::ServiceStartFailure{ref service, ref cause} => format!("Failed to launch commands: {} for {}! Reason: {}", service.clone().start.unwrap_or("#no-commands".to_string()), service, cause),

            &Mortal::NotificationFailure{ref cause} => format!("Failed to send notification! Reason: {}", cause),
        })
    }
}
