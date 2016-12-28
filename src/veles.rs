use std::thread::sleep;
use std::time::Duration;
use std::process::{Command, Stdio};
use std::os::unix::process::CommandExt;
use users::{get_user_by_name, get_group_by_name};

use common::*;
use service::Service;


/*
 * Veles is a service spawner deity
 */
pub trait Veles {

    fn start_service(&self) -> Result<u32, String>;

}


impl Veles for Service {

    fn start_service(&self) -> Result<u32, String> {
        let mut cmd = Command::new(DEFAULT_SHELL);
        cmd.arg("-c");
        match self.start {
            Some(ref commands) => {
                trace!("Built command line: {:?}", commands);
                // cmd.args(commands.as_slice());
                cmd.current_dir(self.work_dir());

                // NOTE: always set stdin to null:
                cmd.stdin(Stdio::null());
                cmd.stdout(Stdio::null());
                cmd.stderr(Stdio::null());

                match get_user_by_name(self.user().as_ref()) {
                    Some(uid) => {
                        trace!("Setting service UID of valid user: {:?}", uid.uid());
                        cmd.uid(uid.uid());
                    },
                    None => {
                        debug!("Username {} not found in system!", self.user())
                    }
                }

                match get_group_by_name(self.group().as_ref()) {
                    Some(gid) => {
                        trace!("Setting service GID of valid group: {:?}", gid.name());
                        cmd.gid(gid.gid());
                    },
                    None => {
                        debug!("Username {} not found in system!", self.group())
                    }
                }

                match cmd.spawn() {
                    Ok(child) => {
                        let pid = child.id();
                        // match child.wait() {}
                        sleep(Duration::from_millis(100));
                        debug!("Service got pid: {}", pid);
                        Ok(pid)
                    }
                    Err(e) => {
                        error!("Failed to spawn command: {:?}. Reason: {}", cmd, e);
                        Err("Failed".to_string())
                    }
                }
            },
            None =>
                Err(format!("No start commands defined for service: {}", self)),
        }
    }
}
