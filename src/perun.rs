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
        let path = self.clone().pid_file.unwrap();
        match Service::load_raw(path.clone()) {
            Ok(raw_content) => {
                let content = raw_content.trim();
                match content.parse::<i32>() {
                    Ok(pid) => unsafe {
                        match kill(pid, 0) {
                            0 => Ok(format!("Service pid: {} from file: {}, is alive!", pid, path)),
                            _ => Err(format!("Service pid: {} from file: {}, seems to be dead!", pid, path))
                        }
                    },
                    Err(cause) =>
                        Err(format!("Service pid from: {}, seems to be malformed: {}! Reason: {:?}", path, content, cause))
                }
            },
            Err(cause) =>
                Err(format!("Service has no pid file: {}! Reason: {:?}", path, cause)),
        }
    }


    fn try_unix_socket(&self) -> Result<String, String> {
        let path = self.clone().unix_socket.unwrap();
        match UnixStream::connect(path.clone()) {
            Ok(mut stream) => {
                match stream.write_all(b"version") {
                    Err(cause) =>
                        Err(format!("Service is not listening on UNIX socket: {}! Reason: {:?}", path, cause)),

                    Ok(_) => {
                        // let mut response = String::new();
                        // stream.read_to_string(&mut response).unwrap();
                        Ok(format!("Service is listening on UNIX socket: {}", path))
                    },
                }
            },
            Err(cause) =>
                Err(format!("Service has no UNIX socket: {}! Reason: {:?}", path, cause)),
        }
    }


    fn checks_for(&self) -> Result<String, String> {
        debug!("Perun::checks_for: {:?}", self);

        // perform Service checks:
        match self.unix_socket.clone() {
            Some(unix_socket) => {
                match self.try_unix_socket() {
                    Ok(_) => {
                        debug!("UNIX socket check passed for Service: {}, with unix_socket: {}", self.name(), unix_socket);
                    },
                    Err(err) => return Err(err),
                }
            },
            None => {},
        }

        match self.pid_file.clone() {
            Some(pid_file) => {
                match self.try_pid_file() {
                    Ok(_) => {
                        debug!("PID check passed for Service: {}, with pid_file: {}", self.name(), pid_file);
                    },
                    Err(err) => return Err(err),
                }
            },
            None => {},
        }

        Ok(format!("All checks passed for service: {}", self.name()))
    }

}
