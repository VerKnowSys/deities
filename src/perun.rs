use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use libc::*;
use colored::*;

use service::Service;


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
                            0 => Ok(format!("Service: {}, pid: {}, from file: {}, is alive!", self, pid, path)),
                            _ => Err(format!("Service: {}, pid: {}, from file: {}, seems to be dead!", self, pid, path))
                        }
                    },
                    Err(cause) =>
                        Err(format!("Service: {}, pid from:, {}, seems to be malformed: {}! Reason: {}", self, self.pid_file(), content, cause))
                }
            },
            Err(cause) =>
                Err(format!("Service: {}, has no pid, file: {}! Reason: {}", self, self.pid_file(), cause)),
        }
    }


    fn try_unix_socket(&self) -> Result<String, String> {
        let path = self.clone().unix_socket();
        match UnixStream::connect(path.clone()) {
            Ok(mut stream) => {
                match stream.write_all(b"version") {
                    Err(cause) =>
                        Err(format!("Service {}, is not listening on UNIX socket: {}! Reason: {:?}", self, self.unix_socket(), cause.kind())),

                    Ok(_) => {
                        // let mut response = String::new();
                        // stream.read_to_string(&mut response).unwrap();
                        Ok(format!("Service {}, is listening on UNIX socket: {}", self, self.unix_socket()))
                    },
                }
            },
            Err(cause) =>
                Err(format!("Service: {} has missing UNIX socket: {}! Reason: {:?}", self, self.unix_socket(), cause.kind())),
        }
    }


    fn checks_for(&self) -> Result<String, String> {
        let mut checks_performed = 0;

        match self.unix_socket().as_ref() {
            "" => trace!("Undefined unix socket for: {}", self),
            _  =>
                match self.try_unix_socket() {
                    Ok(_) => {
                        checks_performed += 1;
                        debug!("UNIX socket check passed for Service: {}, with unix_socket: {}", self, self.unix_socket())
                    },
                    Err(err) => return Err(err),
                },
        }

        match self.pid_file().as_ref() {
            "" => trace!("Undefined unix socket for: {}", self),
            _  =>
                match self.try_pid_file() {
                    Ok(_) => {
                        checks_performed += 1;
                        debug!("PID check passed for Service: {}, with pid_file: {}", self, self.pid_file())
                    },
                    Err(err) => return Err(err),
                },
        }

        trace!("performed {} checks for: {}", checks_performed, self);
        let plu = match checks_performed {
            1 => "check",
            _ => "checks",
        };
        match checks_performed {
            0 => Ok(format!("Ok ⇒ No {} for service: {}", plu, self)),
            _ => Ok(format!("Ok ⇒ {} {} passed for service: {}", format!("{:2}", checks_performed).bold(), plu, self)),
        }
    }

}
