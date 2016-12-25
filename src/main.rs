#[macro_use]
extern crate log;
extern crate colored;
extern crate fern;
extern crate libc;
extern crate rustc_serialize;
extern crate toml;
extern crate glob;
extern crate time;
extern crate uuid;

pub mod common;
pub mod service;
pub mod svarog;
pub mod perun;
pub mod veles;

use std::time::Duration;
use std::thread::sleep;
use colored::*;
use log::LogLevel::*;
use std::collections::HashSet;

use toml::decode_str;
use service::Service;
use veles::Veles;
use perun::Perun;
use common::*;


fn init_logger() {
    let logger_config = fern::DispatchConfig {
        format: Box::new(|message: &str, log_level: &log::LogLevel, _location: &log::LogLocation| {
            // This is a fairly simple format, though it's possible to do more complicated ones.
            // This closure can contain any code, as long as it produces a String message.
            let tim = time::now().strftime("%Y-%m-%d %H:%M:%S").unwrap().to_string().black().bold().dimmed();
            let (lev, msg) = match log_level {
                &Error => (log_level.to_string().red().underline().dimmed(), message.red().bold()),
                &Warn => (log_level.to_string().yellow().underline().dimmed(), message.yellow().bold()),
                &Info => (log_level.to_string().white().underline().dimmed(), message.white()),
                &Debug => (log_level.to_string().cyan().underline().dimmed(), message.cyan()),
                &Trace => (log_level.to_string().magenta().underline().dimmed(), message.magenta()),
            };
            format!("{} {:5} {}", tim, lev, msg)
        }),
        output: vec![fern::OutputConfig::stdout()], // , fern::OutputConfig::file("output.log")
        level: log::LogLevelFilter::Info,
    };
    let _ = fern::init_global_logger(logger_config, log::LogLevelFilter::Info);
}


fn main() {
    init_logger();

    info!("{} v{}", NAME.green().bold(), VERSION.yellow().bold());

    let mut services = HashSet::new();
    let mut services_err = HashSet::new();
    let mut cycle_count = 0u64;
    debug!("{}. Service check interval: {:4}ms", "Veles".green().bold(), CHECK_INTERVAL);

    loop {
        debug!("");
        cycle_count += 1;
        trace!("{} - {}", "check iteration".yellow(), format!("{:06}", cycle_count).yellow().bold());

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
                                                Ok(ok) => {
                                                    services_err.remove(&service.name());
                                                    services.insert(service.name());
                                                    debug!("{}", ok)
                                                },

                                                Err(error) => {
                                                    services.remove(&service.name());
                                                    services_err.insert(service.name());
                                                    error!("{}", error)
                                                }
                                            }
                                        },
                                        None => {
                                            error!("Failed to load service file: {:?}. Please double check definition syntax since we're not validating it properly for now.", service_definition_file)
                                        }
                                    }
                                },
                                Err(error) => {
                                    error!("Definition load failure: {:?}", error)
                                }
                            }
                        },
                        None => error!("No access to definition file! {:?}", path)
                    }
                },
                None => error!("No access to read service definition file?")
            }
        }

        /* Handle adding service definition to monitored dir */
        if Veles::list_services().count() > services.len() + services_err.len() {
            debug!("Resetting service counters after detected changes in definitions ({} > {} + {})", Veles::list_services().count(), services.len(), services_err.len());
            services.clear();
            services_err.clear();
        }

        /* Handle removing service definition from monitored dir */
        if Veles::list_services().count() < services.len() + services_err.len() {
            debug!("Resetting service counters after detected changes in definitions ({} < {} + {})", Veles::list_services().count(), services.len(), services_err.len());
            services.clear();
            services_err.clear();
        }

        let errored = if services_err.len() > 0 {
            format!(" /  {} {} {}",
                "Service failures".red(),
                format!("{:03}", services_err.len()).red().bold(),
                format!("{:?}", services_err).red().italic())
        } else {
            "".to_string()
        };

        info!("{} - {} {} {}",
            "Monitoring services".green(),
            format!("{:03}", services.len()).green().bold(),
            format!("{:?}", services).green().italic(),
            errored
        );

        sleep(Duration::from_millis(CHECK_INTERVAL));
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
