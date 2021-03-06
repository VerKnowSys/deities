use std::{
    fs::{set_permissions, File},
    io::prelude::*,
    os::unix::{fs::PermissionsExt, process::CommandExt},
    process::{Command, Stdio},
};
use users::{get_group_by_name, get_user_by_name};

use crate::{
    common::*,
    init_fields::InitFields,
    mortal::Mortal::{self, *},
    service::Service,
    *,
};


// Veles is a service spawner deity
//
pub trait Veles {
    fn create_shell_wrapper(&self, commands: String) -> String;

    fn start_service(&self) -> Result<u32, Mortal>;
}


impl Veles for Service {
    #[instrument]
    fn create_shell_wrapper(&self, commands: String) -> String {
        let wrapper = format!("{}/.{}.sh", SERVICES_DIR, self.name());
        match File::create(wrapper.clone()) {
            Ok(mut file) => {
                match file
                    .write(format!("#!/bin/sh\nexport PATH={}\n", DEFAULT_PATH).as_bytes())
                {
                    Ok(_) => {
                        // If cleanup routines defined, inject it before spawn:
                        match self.clone().cleanup {
                            Some(cleanup) => {
                                let cl =
                                    format!("\n#Pre-start cleanup routine:\n{}\n\n", cleanup);
                                match file.write(cl.as_bytes()) {
                                    Ok(_) => trace!("Cleanup routine written successfully"),
                                    Err(we) => error!("Cleanup write error!. Reason: {}", we),
                                }
                            }
                            None => trace!("No cleanup routine to inject. Skipped."),
                        }

                        match file.write(commands.as_bytes()) {
                            Ok(_) => {
                                match file.flush() {
                                    Ok(_) => trace!("Flushed successfully"),
                                    Err(fe) => error!("Flush failed! Reason: {}", fe),
                                }
                            }
                            Err(we) => error!("Write2 error!. Reason: {}", we),
                        }
                    }
                    Err(we) => error!("Write1 error!. Reason: {}", we),
                }
                match file.metadata() {
                    Ok(metadata) => {
                        let mut permissions = metadata.permissions();
                        permissions.set_mode(0o777);
                        match set_permissions(wrapper.clone(), permissions) {
                            Ok(_) => trace!("Wrapper executable bits set for {:?}!", file),
                            Err(fail) => error!("Failure setting permissions: {}", fail),
                        }
                    }
                    Err(e) => error!("Can't set mode! Reason: {}", e),
                }
            }
            Err(e) => error!("create_shell_wrapper: {}", e),
        }
        wrapper
    }


    #[instrument]
    fn start_service(&self) -> Result<u32, Mortal> {
        let mut cmd = Command::new(DEFAULT_SHELL);
        cmd.arg("-c");
        match self.start {
            Some(ref commands) => {
                // NOTE: single command - a wrapper:
                cmd.arg(self.create_shell_wrapper(commands.to_string()));

                cmd.current_dir(self.work_dir());
                trace!(
                    "Built command line: {:?} for working dir: {}",
                    commands,
                    self.work_dir()
                );

                // NOTE: always set stdin to null:
                cmd.stdin(Stdio::null());
                cmd.stdout(Stdio::null());
                cmd.stderr(Stdio::null());

                match get_user_by_name(&self.user()) {
                    Some(uid) => {
                        trace!("Setting service UID of valid user: {:?}", uid.name());
                        cmd.uid(uid.uid());
                    }
                    None => warn!("Username {} not found in system!", self.user()),
                }

                match get_group_by_name(&self.group()) {
                    Some(gid) => {
                        trace!("Setting service GID of valid group: {:?}", gid.name());
                        cmd.gid(gid.gid());
                    }
                    None => warn!("Username {} not found in system!", self.group()),
                }

                match cmd.spawn() {
                    Ok(mut child) => {
                        let pid = child.id();
                        warn!("NOTE: Service is supposed to go in background (daemonize)!");
                        child.wait().expect("Service should spawn in background!");
                        Ok(pid)
                    }
                    Err(e) => {
                        error!("Failed to spawn commands: {:?}. Reason: {}", cmd, e);
                        Err(ServiceStartFailure {
                            service: self.clone(),
                            cause: e,
                        })
                    }
                }
            }
            None => {
                Err(ServiceNoStartDefined {
                    service: self.clone(),
                })
            }
        }
    }
}
