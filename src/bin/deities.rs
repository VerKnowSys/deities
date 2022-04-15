use colored::*;
use fs2::FileExt;
use glob::{glob, Paths};
use std::{
    env,
    fs::File,
    path,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread::{self, sleep, Builder},
    time::Duration,
};
use tracing_subscriber::{fmt, EnvFilter};
use users::{Users, UsersCache};
use uuid::Uuid;

// use users::os::unix::{UserExt, GroupExt};
// use users::os::bsd::UserExt as BSDUserExt;

use deities::{
    common::*, init_fields::*, mortal::Mortal, perun::Perun, service::Service, svarog::Svarog,
    veles::Veles, *,
};


/// Initialize logger and tracingformatter
#[instrument]
fn initialize() {
    let env_log = match EnvFilter::try_from_env("LOG") {
        Ok(env_value_from_env) => env_value_from_env,
        Err(_) => EnvFilter::from("info"),
    };
    fmt()
        .compact()
        .with_thread_names(false)
        .with_thread_ids(false)
        .with_ansi(true)
        .with_env_filter(env_log)
        .with_filter_reloading()
        .init();
}


#[instrument]
fn list_services() -> Paths {
    glob(&format!("{}/{}", SERVICES_DIR, SERVICES_GLOB))
        .unwrap_or_else(|_| panic!("Failed to match {}/{}", SERVICES_DIR, SERVICES_GLOB))
}


#[instrument]
fn spawn_thread(service_to_monitor: Result<path::PathBuf, glob::GlobError>) {
    debug!(
        "Thread UUID: {}",
        thread::current()
            .name()
            .unwrap_or(&Uuid::new_v4().to_string())
            .bold()
    );
    match service_to_monitor.unwrap().file_name() {
        Some(path) => {
            match path.to_str() {
                Some(service_definition_file) => {
                    match Service::from(service_definition_file.to_string()) {
                        // perfom Perun checks on service definition:
                        Ok(service) => {
                            let interval = service.checks_interval();
                            debug!("Checks interval: {} ms, of {}", interval, service);
                            sleep(Duration::from_millis(interval));

                            match service.checks_for() {
                                Ok(ok) => info!("{}", ok),

                                /* Handle disk space check without any following action */
                                Err(Mortal::CheckDiskSpace {
                                    service,
                                }) => {
                                    warn!(
                                        "Service requires: {} MiB free disk space!",
                                        service.disk_minimum_space() / 1024
                                    );
                                    match service.notification(
                                        format!(
                                            "Service requires: {} MiB free",
                                            service.disk_minimum_space() / 1024
                                        ),
                                        "Disk space check failure!".to_string(),
                                    ) {
                                        Ok(msg) => {
                                            debug!("Done notification. Result: {}", msg)
                                        }
                                        Err(er) => error!("Error with notification: {}", er),
                                    }
                                }

                                /* Handle disk inodes check without any following action */
                                Err(Mortal::CheckDiskInodes {
                                    service,
                                }) => {
                                    warn!(
                                        "Service requires: {} free inodes!",
                                        service.disk_minimum_inodes()
                                    );
                                    match service.notification(
                                        format!(
                                            "Service requires: {} free inodes",
                                            service.disk_minimum_inodes()
                                        ),
                                        "Disk inodes check failure!".to_string(),
                                    ) {
                                        Ok(msg) => {
                                            debug!("Done notification. Result: {}", msg)
                                        }
                                        Err(er) => error!("Error with notification: {}", er),
                                    }
                                }

                                /*
                                    NOTE: for other types of failures, we want to handle cleanup/ start routines:
                                */
                                Err(error) => {
                                    warn!(
                                        "Detected malfunction of: {}. Reason: {}",
                                        service, error
                                    );
                                    match service.notification(
                                        format!("Detected malfunction of: {}", service),
                                        error.to_string(),
                                    ) {
                                        Ok(msg) => debug!("Notification sent: {}", msg),
                                        Err(er) => error!("{}", er),
                                    }

                                    /* notification sent, now try handling service process */
                                    match service.start_service() {
                                        Ok(_) => {
                                            info!(
                                                "Service started: {}",
                                                service.name().green().bold()
                                            )
                                        }
                                        Err(cause) => {
                                            error!(
                                                "Failed to start service. Reason: {}",
                                                cause
                                            )
                                        }
                                    }
                                }
                            }
                        }

                        Err(reason) => error!("Definition load failure: {:?}", reason),
                    }
                }

                None => error!("Unable to open service file in path: {:?}", path),
            }
        }

        None => error!("No access to read service definition file?"),
    }
}


#[instrument]
fn eternity() -> () {
    let cycle_count = Arc::new(AtomicUsize::new(0));
    loop {
        cycle_count.fetch_add(1, Ordering::SeqCst);
        debug!(
            "Iteration no. {}",
            format!("{}", cycle_count.clone().load(Ordering::SeqCst))
                .yellow()
                .bold()
        );

        // let handlers: Vec<thread::JoinHandle<_>> =
        let out: String = list_services()
            .flat_map(|service_to_monitor| {
                let thread_builder = Builder::new().name(Uuid::new_v4().to_string());
                thread_builder.spawn(|| spawn_thread(service_to_monitor))
            })
            .map(|handle| {
                let name = format!("{}", handle.thread().name().unwrap_or("Unnamed"));
                (handle, name)
            })
            .map(|(handle, name)| {
                match handle.join() {
                    Ok(_) => {
                        format!(
                            "Thread: {} joined iteration: {}",
                            name,
                            cycle_count.load(Ordering::SeqCst)
                        )
                    }
                    Err(cause) => {
                        format!(
                            "Thread: {} failed to join iteration: {}! Internal cause: {:?}",
                            name,
                            cycle_count.load(Ordering::SeqCst),
                            cause
                        )
                    }
                }
            })
            .collect();

        match &out[..] {
            "" => {
                debug!("No services found under /Services, throttling..");
                sleep(Duration::from_millis(CHECKS_INTERVAL))
            }
            traced => trace!("{}", traced),
        }
    }
}


#[instrument]
fn main() {
    initialize();

    let users = UsersCache::new();
    let lock_name = match users.get_current_uid() {
        0 => DEFAULT_LOCK.to_string(),
        _ => {
            format!(
                "{}{}",
                env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()),
                DEFAULT_LOCK
            )
        }
    };

    let lockfile = match File::open(lock_name.clone()) {
        Ok(file) => file,
        Err(_) => {
            match File::create(lock_name.clone()) {
                Ok(file) => file,
                Err(cause) => {
                    error!("Lock creation error: {}", cause);
                    unsafe {
                        libc::exit(libc::EPERM);
                    }
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
        }
    }

    info!("{} v{}", NAME.green().bold(), VERSION.yellow().bold());
    eternity()
}
