#[macro_use]
extern crate log;
extern crate colored;
extern crate fern;
extern crate libc;
extern crate rustc_serialize;
extern crate toml;
extern crate glob;
extern crate time;


pub mod service;
pub mod svarog;
pub mod perun;
pub mod veles;

use toml::decode_str;
use service::Service;
use veles::Veles;
use perun::Perun;

use std::time::Duration;
use std::thread::sleep;
use colored::*;
use log::LogLevel::*;



fn main() {
    env_logger::init().unwrap();

    info!("Veles spawned!");
    loop {
        debug!("\n");
        for service_to_monitor in Veles::list_services() {
            match service_to_monitor.unwrap().file_name() {
                Some(path) => {
                    match path.to_str() {
                        Some(service_definition_file) => {
                            match Service::load(service_definition_file.to_string()) {
                                Ok(service_definition) => {
                                    let service_config: Option<Service> = decode_str(service_definition.as_ref());
                                    match service_config {
                                        Some(service) => {
                                            // perfom Perun checks
                                            match service.checks_for() {
                                                Ok(ok) =>
                                                    info!("{:?}", ok),

                                                Err(error) =>
                                                    error!("{:?}", error),
                                            }
                                        },
                                        None =>
                                            error!("Failed to load service file: {:?}. Please double check definition syntax since we're not validating it properly for now.", service_definition_file)
                                    }
                                },
                                Err(error) => error!("Error {:?}", error)
                            }
                        },
                        None => error!("No file! {:?}", path)
                    }
                },
                None => error!("No access to read service definition file?")
            }
        }
        sleep(Duration::from_millis(2000));
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
