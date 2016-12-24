#[macro_use]
extern crate log;
extern crate env_logger;
extern crate libc;
extern crate rustc_serialize;
extern crate toml;
extern crate glob;


mod common;
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
// use log::LogLevel;


fn main() {
    env_logger::init().unwrap();

    info!("Veles appeared!");
    loop {
        debug!("\n");
        for svce in Veles::list_services() {
            match svce.unwrap().file_name() {
                Some(astr) => {
                    match astr.to_str() {
                        Some(service_file) => {
                            match Service::load(service_file.to_string()) {
                                Ok(service_ref) => {
                                    let val: Option<Service> = decode_str(service_ref.as_ref());
                                    match val {
                                        Some(service) => {
                                            // perfom Perun checks
                                            match service.checks_for() {
                                                Ok(ok) =>
                                                    info!("{:?}", ok),

                                                Err(error) =>
                                                    error!("{:?}", error),
                                            }
                                        },
                                        None => {
                                            error!("Failed to load service file: {:?}. Please double check definition syntax since we're not validating it properly for now.", service_file);
                                        }
                                    }
                                },
                                Err(error) => {
                                    error!("Error {:?}", error);
                                }
                            }
                        },
                        None => {
                            error!("No file?");
                        }
                    }
                },
                None => {
                    error!("No access to read file?");
                }
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
