use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use libc::*;
use colored::*;

use curl::easy::Easy;

use common::*;
use service::Service;


/*
 * Perun is a supervisor deity
 */
pub trait Perun {
    fn try_pid_file(&self) -> Result<String, String>;
    fn try_unix_socket(&self) -> Result<String, String>;
    fn try_urls(&self) -> Result<String, String>;

    fn checks_for(&self) -> Result<String, String>;
}


impl Perun for Service {


    fn try_urls(&self) -> Result<String, String> {
        for url in self.urls() {
            // let mut dst = Vec::new();
            let mut easy = Easy::new();
            trace!("Making request to: {} for: {}", url, self.bold());
            match easy.url(url.as_ref()) {
                Ok(_) =>
                    match easy.perform() {
                        Ok(_) => {},
                        Err(cause) =>
                            return Err(format!("Failure: {}, while checking URL: {} for: {}", cause, url, self.bold())),
                    },
                Err(error) =>
                    return Err(format!("URL: {} failed: {} for: {}", url, error, self.bold()))
            }
        }
        let urls_to_ch = match self.urls().len() {
            1 => "url",
            _ => "urls",
        };
        Ok(format!("Ok - {} {} successfully checked for: {}", self.urls().len(), urls_to_ch, self.bold()))
    }


    fn try_pid_file(&self) -> Result<String, String> {
        let path = self.clone().pid_file();
        match Service::load_raw(path.clone()) {
            Ok(raw_content) => {
                let content = raw_content.trim();
                match content.parse::<i32>() {
                    Ok(pid) => unsafe {
                        match kill(pid, 0) {
                            0 => Ok(format!("{}, pid: {}, from file: {}, is alive!", self.bold(), pid, path)),
                            _ => Err(format!("{}, pid: {}, from file: {}, seems to be dead!", self.bold(), pid, path))
                        }
                    },
                    Err(cause) =>
                        Err(format!("{}, pid from:, {}, seems to be malformed: {}! Reason: {}", self.bold(), self.pid_file(), content, cause))
                }
            },
            Err(cause) =>
                Err(format!("{}, has no pid, file: {}! Reason: {}", self.bold(), self.pid_file(), cause)),
        }
    }


    fn try_unix_socket(&self) -> Result<String, String> {
        let path = self.clone().unix_socket();
        match UnixStream::connect(path.clone()) {
            Ok(mut stream) => {
                match stream.write_all(UNIX_SOCKET_MSG) {
                    Err(cause) =>
                        Err(format!("{}, is not listening on UNIX socket: {}! Reason: {:?}", self.bold(), self.unix_socket(), cause.kind())),

                    Ok(_) => {
                        // let mut response = String::new();
                        // stream.read_to_string(&mut response).unwrap();
                        Ok(format!("{}, is listening on UNIX socket: {}", self.bold(), self.unix_socket()))
                    },
                }
            },
            Err(cause) =>
                Err(format!("Missing UNIX socket: {} for: {}! Reason: {:?}", self.unix_socket(), self.bold(), cause.kind())),
        }
    }


    fn checks_for(&self) -> Result<String, String> {
        let mut checks_performed = 0;

        match self.unix_socket().as_ref() {
            "" => trace!("Undefined unix_socket for: {}", self.bold()),
            _  =>
                match self.try_unix_socket() {
                    Ok(_) => {
                        checks_performed += 1;
                        debug!("UNIX socket check passed for: {}, with unix_socket: {}", self.bold(), self.unix_socket())
                    },
                    Err(err) => return Err(err),
                },
        }

        match self.pid_file().as_ref() {
            "" => trace!("Undefined pid_file for: {}", self.bold()),
            _  =>
                match self.try_pid_file() {
                    Ok(_) => {
                        checks_performed += 1;
                        debug!("PID check passed for: {}, with pid_file: {}", self.bold(), self.pid_file())
                    },
                    Err(err) => return Err(err),
                },
        }

        if self.urls().len() > 0 {
            match self.try_urls() {
                Ok(_) => {
                    checks_performed += 1;
                    debug!("URLs check passed for: {}, with urls: {:?}", self.bold(), self.urls())
                },
                Err(err) => return Err(err),
            }
        } else {
            trace!("Undefined urls for: {}", self.bold())
        }


        trace!("performed {} checks for: {}", checks_performed, self.bold());
        let plu = match checks_performed {
            1 => "check",
            _ => "checks",
        };
        match checks_performed {
            0 => Ok(format!("Ok ⇒ No {} for: {}", plu, self.bold())),
            _ => Ok(format!("Ok ⇒ {} {} passed for: {}", format!("{:2}", checks_performed).bold(), plu, self.bold())),
        }
    }

}
