use std::io::prelude::*;
use std::fs::File;
use std::fmt;
use std::fmt::Display;
use colored::*;
use toml::*;
use toml::de::Error as TomlError;
use std::io::{Error, ErrorKind};
use std::env;
use regex::Regex;

use common::*;
use svarog::Svarog;
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

    // watch service availability through pid_file
    pid_file: Option<String>,

    /// watch if service listens is a vector of IP:PORT elements like: ["127.0.0.1:1234", "1.2.3.4:5000"]
    // listens: Option<Vec<String>>,

    /// watch if service domains is a vector of PROTO+FQDN elements like: ["https://my.shiny.domain.com/page2?param=1", "http://some.com"]
    urls: Option<Vec<String>>,

    /// default initialization file of service
    ini_file: Option<String>,
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


    /// returns service name
    pub fn name(&self) -> String {
        match self.name.clone() {
            Some(name) => name,
            None => {
                // cut off file extension and use it as default service name:
                let rx = Regex::new(r"\..*$").unwrap();
                rx.replace_all(self.ini_file().as_ref(), "").to_string()
            },
        }
    }


    pub fn user(&self) -> String {
        match self.user.clone() {
            Some(name) => name,
            None =>
                match env::var("USER") {
                    Ok(somebody) => somebody.to_owned(),
                    Err(_) => "nobody".to_string(),
                },
        }
    }


    pub fn group(&self) -> String {
        match self.group.clone() {
            Some(group) => group,
            None =>
                match self.sys_info().sysname.as_ref() {
                    // on macOS it's better to assume that our user is in staff group:
                    "Darwin" => "staff".to_string(),
                    _ => "nobody".to_string(),
                },
        }
    }


    pub fn styled(&self) -> ColoredString {
        self.to_string().underline().italic()
    }


    /// returns service working dir
    pub fn work_dir(&self) -> String {
        match self.work_dir.clone() {
            Some(path) => path,
            None => "/tmp".to_string(),
        }
    }


    /// returns service pid file to monitor
    pub fn pid_file(&self) -> String {
        match self.pid_file.clone() {
            Some(path) => path,
            None => "".to_string(),
        }
    }


    /// returns path to unix socket to monitor
    pub fn unix_socket(&self) -> String {
        match self.unix_socket.clone() {
            Some(path) => path,
            None => "".to_string(),
        }
    }


    /// returns urls list to check
    pub fn urls(&self) -> Vec<String> {
        match self.urls.clone() {
            Some(vector) => vector,
            None => vec!(),
        }
    }


    pub fn ini_file(&self) -> String {
        match self.ini_file.clone() {
            Some(file_path) => file_path,
            None => "undefined-ini-file".to_string(),
        }
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
