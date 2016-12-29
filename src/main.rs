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
extern crate curl;
extern crate slack_hook;
extern crate chrono;
extern crate hostname;
extern crate uname;
extern crate users;
extern crate fs2;

pub mod common;
pub mod service;
pub mod mortal;
pub mod svarog;
pub mod perun;
pub mod veles;

use std::time::Duration;
use std::thread::sleep;
use colored::*;
use log::LogLevel::*;
use log::LogLevelFilter;
use fern::init_global_logger;
use fern::{DispatchConfig, OutputConfig};
use std::env;
use std::sync::Arc;
use std::thread;
use std::thread::Builder;
use std::sync::atomic::{AtomicUsize, Ordering};
use uuid::Uuid;
use glob::glob;
use glob::Paths;
use std::fs::File;
use fs2::FileExt;
use users::{Users, UsersCache};
// use users::os::unix::{UserExt, GroupExt};
// use users::os::bsd::UserExt as BSDUserExt;


use veles::Veles;
use service::Service;
use perun::Perun;
use svarog::Svarog;
use common::*;


/// initialize internal logger
fn init_logger() {
    let logger = DispatchConfig {
        format: Box::new(|message: &str, log_level: &log::LogLevel, _location: &log::LogLocation| {
            // This is a fairly simple format, though it's possible to do more complicated ones.
            // This closure can contain any code, as long as it produces a String message.
            let tim = time::now().strftime("%Y-%m-%d %H:%M:%S").unwrap().to_string().black().bold().dimmed();
            let (lev, msg) = match log_level {
                &Error => (log_level.to_string().red().underline().dimmed(), message.red().underline()),
                &Warn => (log_level.to_string().yellow().underline().dimmed(), message.yellow().underline()),
                &Info => (log_level.to_string().white().underline().dimmed(), message.white()),
                &Debug => (log_level.to_string().cyan().underline().dimmed(), message.cyan()),
                &Trace => (log_level.to_string().magenta().underline().dimmed(), message.magenta()),
            };
            format!("{} {:5} {}", tim, lev, msg)
        }),
        output: vec![OutputConfig::stdout()], // , fern::OutputConfig::file("output.log")
        level: LogLevelFilter::Trace,
    };

    /* dynamic logger configuration */
    match env::var(LOG_ENV) {
        Ok(val) => {
            match val.as_ref() {
                "trace" =>
                    init_global_logger(logger, LogLevelFilter::Trace).unwrap(),
                "debug" =>
                    init_global_logger(logger, LogLevelFilter::Debug).unwrap(),
                "info" =>
                    init_global_logger(logger, LogLevelFilter::Info).unwrap(),
                "warn" =>
                    init_global_logger(logger, LogLevelFilter::Warn).unwrap(),
                "error" =>
                    init_global_logger(logger, LogLevelFilter::Error).unwrap(),
                _ =>
                    init_global_logger(logger, LogLevelFilter::Info).unwrap(),
            }
        },
        Err(_) => init_global_logger(logger, LogLevelFilter::Info).unwrap(),
    }

}


fn list_services() -> Paths {
    glob(
        &format!("{}/{}", SERVICES_DIR, SERVICES_GLOB)
    ).expect(
        &format!("Failed to match {}/{}", SERVICES_DIR, SERVICES_GLOB)
    )
}


fn main() {
    init_logger();

    let users = UsersCache::new();
    let lock_name = match users.get_current_uid() {
        0 => DEFAULT_LOCK.to_string(),
        _ => format!("{}{}", env::var("HOME").unwrap_or("/tmp".to_string()), DEFAULT_LOCK),
    };

    let lockfile = match File::open(lock_name.clone()) {
        Ok(file) => file,
        Err(_) => match File::create(lock_name.clone()) {
            Ok(file) => file,
            Err(cause) => {
                error!("Lock creation error: {}", cause);
                unsafe {
                    libc::exit(libc::EPERM);
                }
            }
        }
    };

    debug!("Trying for lock file: {}", lock_name);
    match lockfile.try_lock_exclusive() {
        Ok(_) => info!("Lock file acquired: {}", lock_name),
        Err(_) => {
            error!("Lock file already acquired. {} is already running!", NAME);
            unsafe {
                libc::exit(libc::EWOULDBLOCK);
            }
        },
    }

    info!("{} v{}", NAME.green().bold(), VERSION.yellow().bold());
    debug!("{}. Service check interval: {:4}ms", "Veles".green().bold(), CHECK_INTERVAL);

    let cycle_count = Arc::new(AtomicUsize::new(0));
    loop {
        cycle_count.fetch_add(1, Ordering::SeqCst);

        for service_to_monitor in list_services() {
            debug!("Iteration no. {}", format!("{}", cycle_count.clone().load(Ordering::SeqCst)).yellow().bold());

            let thread_builder = Builder::new().name(Uuid::new_v4().to_string());
            let handler = thread_builder.spawn( || {
                debug!("Thread UUID: {}", thread::current().name().unwrap_or(&Uuid::new_v4().to_string()).bold());
                match service_to_monitor.unwrap().file_name() {
                    Some(path) => {
                        match path.to_str() {
                            Some(service_definition_file) => {
                                match Service::new_from(service_definition_file.to_string()) {
                                    // perfom Perun checks on service definition:
                                    Ok(service) => {
                                        match service.checks_for() {
                                            Ok(ok) =>
                                                info!("{}", ok.green()),

                                            Err(error) => {
                                                if SLACK_WEBHOOK_URL == "" {
                                                    warn!("SLACK_WEBHOOK_URL is unset. Slack notifications will NOT be sent!");
                                                } else {
                                                    match service.notification(
                                                        format!("malfunction of: {}", service.to_string()), error.to_string()) {
                                                        Ok(msg) =>
                                                            trace!("Notification sent: {}", msg),
                                                        Err(er) =>
                                                            error!("{}", er),
                                                    }
                                                }
                                                error!("Detected malfunction of: {}. Reason: {}", service.to_string(), error);

                                                // notification sent, now try handling service process
                                                match service.start_service() {
                                                    Ok(_) => info!("Service started: {}", service.name().green().bold()),
                                                    Err(cause) => error!("Failed to start service. Reason: {}", cause),
                                                }

                                            },
                                        }
                                    },

                                    Err(reason) => error!("Definition load failure: {:?}", reason),
                                }
                            },

                            None => error!("Unable to open service file in path: {:?}", path),
                        }
                    },

                    None => error!("No access to read service definition file?")
                }
            });

            match handler {
                Ok(handle) => {
                    match handle.join() {
                        Ok(_) => trace!("Handler is joining threads.."),
                        Err(cause) => error!("Failed joining threads! Cause: {:?}", cause),
                    }
                },
                Err(cause) => error!("Handler failed! Cause: {:?}", cause),
            }
        }

        sleep(Duration::from_millis(CHECK_INTERVAL));
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
