use std::os::unix::net::UnixStream;
use std::io::prelude::*;

use service::Service;
use libc::*;


/*
 * Perun is a supervisor deity
 */
pub trait Perun {
    fn try_pid_file(&self) -> Result<String, String>;
    fn try_unix_socket(&self) -> Result<String, String>;

    fn checks_for(&self) -> Result<String, String>;
}


impl Perun for Service {


    fn try_pid_file(&self) -> Result<String, String> {
        let path = self.clone().pid_file();
        match Service::load_raw(path.clone()) {
            Ok(raw_content) => {
                let content = raw_content.trim();
                match content.parse::<i32>() {
                    Ok(pid) => unsafe {
                        match kill(pid, 0) {
                            0 => Ok(format!("Service: {}, pid: {}, from file: {}, is alive!", self.name(), pid, path)),
                            _ => Err(format!("Service: {}, pid: {}, from file: {}, seems to be dead!", self.name(), pid, path))
                        }
                    },
                    Err(cause) =>
                        Err(format!("Service: {}, pid from:, {}, seems to be malformed: {}! Reason: {}", self.name(), path, content, cause))
                }
            },
            Err(cause) =>
                Err(format!("Service: {}, has no pid, file: {}! Reason: {}", self.name(), path, cause)),
        }
    }


    fn try_unix_socket(&self) -> Result<String, String> {
        let path = self.clone().unix_socket();
        match UnixStream::connect(path.clone()) {
            Ok(mut stream) => {
                match stream.write_all(b"version") {
                    Err(cause) =>
                        Err(format!("Service {}, is not listening on UNIX socket: {}! Reason: {:?}", self.name(), path, cause.kind())),

                    Ok(_) => {
                        // let mut response = String::new();
                        // stream.read_to_string(&mut response).unwrap();
                        Ok(format!("Service {}, is listening on UNIX socket: {}", self.name(), path))
                    },
                }
            },
            Err(cause) =>
                Err(format!("Service: {} has missing UNIX socket: {}! Reason: {:?}", self.name(), path, cause.kind())),
        }
    }


    fn checks_for(&self) -> Result<String, String> {
        trace!("{}", self);

        match self.try_unix_socket() {
            Ok(_) => {
                trace!("UNIX socket check passed for Service: {}, with unix_socket: {}", self.name(), self.unix_socket());
            },
            Err(err) => return Err(err),
        }

        match self.try_pid_file() {
            Ok(_) => {
                trace!("PID check passed for Service: {}, with pid_file: {}", self.name(), self.pid_file());
            },
            Err(err) => return Err(err),
        }

        Ok(format!("All checks passed for service: {}", self))
    }

}
