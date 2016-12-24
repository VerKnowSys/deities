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
    let logger_config = fern::DispatchConfig {
        format: Box::new(|message: &str, log_level: &log::LogLevel, _location: &log::LogLocation| {
            // This is a fairly simple format, though it's possible to do more complicated ones.
            // This closure can contain any code, as long as it produces a String message.
            let tim = time::now().strftime("%Y-%m-%d][%H:%M:%S").unwrap().to_string().black();
            let (lev, msg) = match log_level {
                &Error => (log_level.to_string().red(), message.red().bold()),
                &Warn => (log_level.to_string().yellow(), message.yellow().bold()),
                &Info => (log_level.to_string().white(), message.white()),
                &Debug => (log_level.to_string().cyan(), message.cyan()),
                &Trace => (log_level.to_string().magenta(), message.magenta()),
            };
            format!("[{}] [{:5}] {}", tim, lev, msg)
        }),
        output: vec![fern::OutputConfig::stdout()], // , fern::OutputConfig::file("output.log")
        level: log::LogLevelFilter::Debug,
    };
    let _ = fern::init_global_logger(logger_config, log::LogLevelFilter::Debug);

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
