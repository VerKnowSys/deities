use std::io::prelude::*;
use std::fs::File;
use std::fmt;
use std::fmt::Display;
use colored::*;
use toml::decode_str;

use common::*;


/*
 * Service structure is a generic service representation.
 */


#[derive(RustcDecodable, Debug, Clone)]
pub struct Service {

    /* Veles: */
    name: Option<String>,
    user: Option<String>,
    group: Option<String>,

    /* Svarog: */
    // pub configure: Option<String>,
    // pub start: Option<String>,
    pub start: Option<Vec<String>>,
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
}


impl Service {


    pub fn new_from(file_name: String) -> Result<Service, String> {
        match Service::load_definition(file_name.clone()) {
            Ok(service_definition) => {
                let service_config: Option<Service> = decode_str(service_definition.as_ref());
                match service_config {
                    Some(service) => Ok(service),
                    None => Err(format!("Couldn't load definition from file: {}", file_name))
                }
            },

            Err(cause) => Err(cause),
        }
    }


    /// loads raw file as String
    pub fn load_raw(file_name: String) -> Result<String, String> {
        match File::open(file_name.clone()) {
            Ok(mut file) => {
                let mut buffer = String::new();
                match file.read_to_string(&mut buffer) {
                    Ok(_read_size) => Ok(buffer.to_owned()),
                    Err(error) => Err(format!("Failed to read definition file: {}. Reason: {:?}", file_name, error.kind()))
                }
            },
            Err(cause) => Err(format!("Error loading file: {}. Reason: {:?}", file_name, cause.kind()))
        }
    }


    /// loads service definition from toml (ini) file
    pub fn load_definition(def_name: String) -> Result<String, String> {
        let def_abspath = format!("{}/{}", SERVICES_DIR, def_name.clone());
        match Service::load_raw(def_abspath) {
            Ok(content) => Ok(content),
            Err(error) => Err(error),
        }
    }


    /// returns service name
    pub fn name(&self) -> String {
        match self.name.clone() {
            Some(name) => name,
            None => "".to_string(), /* TODO: use definition file name fallback! */
        }
    }


    pub fn user(&self) -> String {
        match self.user.clone() {
            Some(name) => name,
            None => "nobody".to_string(),
        }
    }


    pub fn group(&self) -> String {
        match self.group.clone() {
            Some(group) => group,
            None => "nogroup".to_string(),
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

}


impl Default for Service {
    fn default() -> Service {
        Service {
            /* Veles: */
            name: None,
            user: None,
            group: None,
            work_dir: None,

            /* Svarog: */
            // configure: None,
            // start: None,
            start: None,
            // after_start: None,
            // stop: None,
            // after_stop: None,
            // reload: None,
            // validate: None,

            /* Perun: */
            conf_file: None,
            unix_socket: None,
            pid_file: None,
            // listens: None,
            urls: None,
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

        write!(f, "Service(name: {}{})", slf.name(), format!("{}{}", optional_pid_entry, optional_sock_entry))
    }
}
