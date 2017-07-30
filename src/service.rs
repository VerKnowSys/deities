use std::io::prelude::*;
use std::fs::File;
use std::fmt;
use std::fmt::Display;
use colored::*;
use toml::*;
use toml::de::Error as TomlError;
use std::io::{Error, ErrorKind};

use common::*;
use init_fields::InitFields;
use mortal::Mortal;
use mortal::Mortal::*;


/*
 * Service structure is a generic service representation.
 */


#[derive(Deserialize, Debug, Clone, Default)]
pub struct Service {

    /* Veles: */
    name: Option<String>,
    user: Option<String>,
    group: Option<String>,

    /* Svarog: */
    // pub configure: Option<String>,
    // pub start: Option<String>,
    pub start: Option<String>,
    pub cleanup: Option<String>,
    // pub after_start: Option<String>,
    // pub stop: Option<String>,
    // pub after_stop: Option<String>,
    // pub reload: Option<String>,
    // pub validate: Option<String>,


    /* Perun: */

    /// service main service configuration file
    conf_file: Option<String>,

    /// determines directory to jump - before starting service
    work_dir: Option<String>,

    // watch service availability through UNIX socket:
    unix_socket: Option<String>,

    disk_space: Option<i64>,
    disk_inodes: Option<i64>,

    // watch service availability through pid_file
    pid_file: Option<String>,

    /// watch if service listens is a vector of IP:PORT elements like: ["127.0.0.1:1234", "1.2.3.4:5000"]
    // listens: Option<Vec<String>>,

    /// watch if service domains is a vector of PROTO+FQDN elements like: ["https://my.shiny.domain.com/page2?param=1", "http://some.com"]
    urls: Option<Vec<String>>,

    /// default initialization file of service
    ini_file: Option<String>,

    /// CHECK_INTERVAL
    check_interval: Option<u64>,

    /// CHECK_URL_TIMEOUT
    check_urltimeout: Option<u64>,

    /// DEATHWATCH_INTERVAL
    deathwatch_interval: Option<u64>,

    /// Slack Notifications can be overridden on service init file
    slack_webhookurl: Option<String>,
    slack_alertchannel: Option<String>,
}


impl Service {


    pub fn new_from(file_name: String) -> Result<Service, Mortal> {
        let def_abspath = format!("{}/{}", SERVICES_DIR, file_name.clone());
        match Service::load_definition(def_abspath) {
            Ok(service_definition) => {
                let service_config: Result<Service, TomlError> = from_str(service_definition.as_ref());
                match service_config {
                    Ok(service) => Ok(Service{ini_file: Some(file_name), .. service}),
                    Err(_) => Err(DefinitionDecodeFailure{ini_name: file_name, cause: Error::new(ErrorKind::Other, "Definition parse error! (detailed parse errors NYD!)".to_string())}),
                }
            },
            Err(cause) => Err(DefinitionLoadFailure{ini_name: file_name, cause: Error::new(ErrorKind::Other, cause.to_string())}),
        }
    }


    /// loads raw file as String
    pub fn load_raw(file_name: String) -> Result<String, Mortal> {
        match File::open(file_name.clone()) {
            Ok(mut file) => {
                let mut buffer = String::new();
                match file.read_to_string(&mut buffer) {
                    Ok(_read_size) => Ok(buffer.to_owned()),
                    Err(error) => Err(RawLoadFailure{file_name: file_name, cause: error}),
                }
            },
            Err(cause) => Err(RawAccessFailure{file_name: file_name, cause: cause}),
        }
    }


    /// loads service definition from toml (ini) file
    pub fn load_definition(abs_path_to_file: String) -> Result<String, Mortal> {
        match Service::load_raw(abs_path_to_file) {
            Ok(content) => Ok(content),
            Err(error) => Err(error),
        }
    }


    pub fn styled(&self) -> ColoredString {
        self.to_string().underline().italic()
    }


}


impl Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let slf = self.clone();

        let optional_pid_entry = match slf.pid_file().as_ref() {
            "" => "".to_string(),
            _  => format!(", pid_file: {}", slf.pid_file()),
        };
        let optional_sock_entry = match slf.unix_socket().as_ref() {
            "" => "".to_string(),
            _  => format!(", unix_socket: {}", slf.unix_socket()),
        };
        let optional_urls_entries = match slf.urls().len() {
            0 => "".to_string(),
            _ => format!(", urls: [{}]", slf.urls().join(", ")),
        };

        write!(f, "{}", format!("Service(name: {}, ini: {}{}{}{})",
            slf.name(),
            slf.ini_file(),
            optional_pid_entry,
            optional_sock_entry,
            optional_urls_entries,
        ))
    }
}
