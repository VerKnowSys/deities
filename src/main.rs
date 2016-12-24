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


fn main() {
    println!("Veles appeared.");

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
                                        println!("{:?}", service);

                                    },

                                    None => {
                                        println!("{:?}", val);
                                    }
                                }
                            },

                            Err(error) => {
                                println!("Error {:?}", error);
                            }
                        }
                    },
                    None => {
                        println!("No file?");
                    }
                }
            },
            None => {
                println!("No access to read file?");
            }
        }

    }
    println!("Veles vanished.");
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
