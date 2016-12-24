use std::io::prelude::*;
use std::fs::File;


/*
 * Service structure is a generic service representation.
 */


#[derive(RustcDecodable, Debug, Clone)]
pub struct Service {

    /* Veles: */
    name: Option<String>,
    dir: Option<String>,


    /* Svarog: */
    pub configure: Option<String>,
    pub start: Option<String>,
    pub after_start: Option<String>,
    pub stop: Option<String>,
    pub after_stop: Option<String>,
    pub reload: Option<String>,
    pub validate: Option<String>,


    /* Perun: */

    // watch service availability through UNIX socket:
    pub unix_socket: Option<String>,

    // watch service availability through pid_file
    pub pid_file: Option<String>,

    /// watch if service listens is a vector of URLs like: ["127.0.0.1:1234", "1.2.3.4:5000"]
    pub listens: Option<Vec<String>>,

    /// watch if service domains is a vector of FQDN like: ["my.shiny.domain.com/page2?param=1", "some.com"]
    pub domains: Option<Vec<String>>,
}


impl Service {


    pub fn name(&self) -> String {
        self.name.clone().unwrap_or(String::from("Unnamed Service"))
    }


    pub fn dir(&self) -> String {
        self.dir.clone().unwrap_or(String::from("/tmp"))
    }


    /// loads service definition from toml (ini) file
    pub fn load(name: String) -> Result<String, String> {
        let file_name = format!("/Services/{}", name.clone());
        match Service::load_raw(file_name) {
            Ok(content) => Ok(content),
            Err(error) => Err(error),
        }
    }


    pub fn load_raw(file_name: String) -> Result<String, String> {
        match File::open(file_name.clone()) {
            Ok(mut file) => {
                let mut buffer = String::new();
                match file.read_to_string(&mut buffer) {
                    Ok(_read_size) => {
                        Ok(buffer.to_owned())
                    },
                    Err(error) => {
                        Err(format!("Failed to read definition: {:?} {:?}", file, error))
                    }
                }
            },
            Err(cause) => {
                Err(format!("Err for file: {:?}, cause: {:?}", file_name, cause))
            }
        }
    }

}


impl Default for Service {
    fn default() -> Service {
        Service {
            /* Veles: */
            name: None,
            dir: None,

            /* Svarog: */
            configure: None,
            start: None,
            after_start: None,
            stop: None,
            after_stop: None,
            reload: None,
            validate: None,

            /* Perun: */
            unix_socket: None,
            pid_file: None,
            listens: None,
            domains: None,
        }
    }
}
