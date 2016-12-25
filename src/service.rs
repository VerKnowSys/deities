use std::io::prelude::*;
use std::fs::File;
use std::fmt;
use std::fmt::Display;
use colored::*;

use common::*;


/*
 * Service structure is a generic service representation.
 */


#[derive(RustcDecodable, Debug, Clone)]
pub struct Service {

    /* Veles: */
    name: Option<String>,
    work_dir: Option<String>,


    /* Svarog: */
    // pub configure: Option<String>,
    // pub start: Option<String>,
    // pub after_start: Option<String>,
    // pub stop: Option<String>,
    // pub after_stop: Option<String>,
    // pub reload: Option<String>,
    // pub validate: Option<String>,


    /* Perun: */

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


    /// returns service name
    pub fn name(&self) -> String {
        match self.name.clone() {
            Some(name) => name,
            None => "".to_string(), /* TODO: use definition file name fallback! */
        }
    }


    pub fn styled(&self) -> ColoredString {
        self.to_string().underline().italic()
    }


    /// returns service working dir
    pub fn dir(&self) -> String {
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


    /// loads service definition from toml (ini) file
    pub fn load(name: String) -> Result<String, String> {
        let file_name = format!("{}/{}", SERVICES_DIR, name.clone());
        match Service::load_raw(file_name) {
            Ok(content) => Ok(content),
            Err(error) => Err(error),
        }
    }


    /// loads raw file as String
    pub fn load_raw(file_name: String) -> Result<String, String> {
        match File::open(file_name.clone()) {
            Ok(mut file) => {
                let mut buffer = String::new();
                match file.read_to_string(&mut buffer) {
                    Ok(_read_size) => Ok(buffer.to_owned()),
                    Err(error) => Err(format!("Failed to read definition: {:?} {:?}", file, error))
                }
            },
            Err(cause) => Err(format!("Err for file: {:?}, cause: {:?}", file_name, cause))
        }
    }

}


impl Default for Service {
    fn default() -> Service {
        Service {
            /* Veles: */
            name: None,
            work_dir: None,

            /* Svarog: */
            // configure: None,
            // start: None,
            // after_start: None,
            // stop: None,
            // after_stop: None,
            // reload: None,
            // validate: None,

            /* Perun: */
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
