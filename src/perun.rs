use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use colored::*;
use curl::easy::Easy;
use std::time::Duration;
use std::path::Path;
use std::io::{Error, ErrorKind};
use std::thread::sleep;
use libc;
use libc::kill;

use common::*;
use service::Service;
use mortal::Mortal;
use mortal::Mortal::*;


/*
 * Perun is a supervisor deity
 */
pub trait Perun {

    /// death_watch will kill service gracefully in case of failure
    /// instead of killing forcefully (kill -9)
    fn death_watch(&self, signal: libc::c_int) -> Result<Mortal, Mortal>;

    fn pid(&self) -> i32;

    fn read_pid(&self) -> Result<i32, Mortal>;
    fn try_pid_file(&self) -> Result<Mortal, Mortal>;
    fn try_unix_socket(&self) -> Result<String, Mortal>;
    fn try_urls(&self) -> Result<String, Mortal>;

    fn checks_for(&self) -> Result<String, Mortal>;
}


impl Perun for Service {


    fn try_urls(&self) -> Result<String, Mortal> {
        for url in self.urls() {
            // let mut dst = Vec::new();
            let mut easy = Easy::new();
            easy.connect_timeout(Duration::from_millis(URLCHECK_TIMEOUT)).unwrap();
            easy.timeout(Duration::from_millis(URLCHECK_TIMEOUT)).unwrap();
            easy.dns_cache_timeout(Duration::from_millis(URLCHECK_TIMEOUT)).unwrap();
            easy.tcp_nodelay(true).unwrap();
            easy.follow_location(true).unwrap();
            easy.ssl_verify_host(true).unwrap();
            easy.ssl_verify_peer(true).unwrap();
            easy.cainfo(Path::new(CACERT_PEM)).unwrap();
            match easy.url(url.as_ref()) {
                Ok(_) =>
                    match easy.perform() {
                        Ok(_) => trace!("Done request to: {} for: {}", url, self.styled()),
                        Err(cause) => return Err(CheckURL{service: self.clone(), url: url, cause: cause}),
                    },
                Err(cause) => return Err(CheckURLFail{service: self.clone(), cause: cause}),
            }
        }
        let urls_to_ch = match self.urls().len() {
            1 => "url",
            _ => "urls",
        };
        Ok(format!("Ok - {} {} successfully checked for: {}", self.urls().len(), urls_to_ch, self.styled()))
    }


    fn death_watch(&self, signal: libc::c_int) -> Result<Mortal, Mortal> {
        let pid = match self.pid() {
           -1 => return Err(SanityCheckFailure{message: "Invalid pid: -1!".to_string()}),
            0 => return Err(SanityCheckFailure{message: "Given pid: 0, it usually means that no process to kill, cause it's already dead.".to_string()}),
            1 => return Err(SanityCheckFailure{message: "You can't put a death watch on pid: 1!".to_string()}),
            any => any,
        };

        unsafe {
            if kill(pid, 0) == 0 {
                trace!("Process with pid: {}, still exists in process list! Perun enters the room!", pid);
                if signal != libc::SIGCONT {
                    sleep(Duration::from_millis(DEFAULT_DEATHWATCH_INTERVAL))
                }
                if kill(pid, signal) == 0 {
                    if kill(pid, 0) != 0 {
                        debug!("Process with pid: {}, was interruped!", pid);
                        return Ok(OkPidInterrupted{service: self.clone(), pid: pid})
                    }
                }
                match signal {
                    libc::SIGCONT => self.death_watch(libc::SIGINT),
                    libc::SIGINT => self.death_watch(libc::SIGTERM),
                    libc::SIGTERM => self.death_watch(libc::SIGKILL),
                    libc::SIGKILL => self.death_watch(libc::SIGKILL),
                    any => Err(SanityCheckFailure{message: format!("Unhandled death_watch signal: {}", any)}),
                }
            } else {
                Err(OkPidAlreadyInterrupted{service: self.clone(), pid: pid})
            }
        }
    }


    fn pid(&self) -> i32 {
        match self.read_pid() {
            Ok(pid) => pid,
            Err(_) => -1,
        }
    }


    fn read_pid(&self) -> Result<i32, Mortal> {
        match Service::load_raw(self.clone().pid_file()) {
            Ok(raw_content) => {
                let content = raw_content.trim();
                match content.parse::<i32>() {
                    Ok(pid) => Ok(pid),
                    Err(_) => Err(CheckPidfileMalformed{service: self.clone()}),
                }
            },
            Err(cause) => Err(CheckPidfileUnaccessible{service: self.clone(), cause: Error::new(ErrorKind::PermissionDenied, cause.to_string())}),
        }
    }


    fn try_pid_file(&self) -> Result<Mortal, Mortal> {
        match self.read_pid() {
            Ok(pid) =>
                unsafe {
                    match kill(pid, 0) {
                        0 => Ok(OkPidAlive{service: self.clone(), pid: pid}),
                        _ => Err(CheckPidfileMalformed{service: self.clone()}),
                    }
                },
            Err(err) => Err(err),
        }
    }


    fn try_unix_socket(&self) -> Result<String, Mortal> {
        let path = self.clone().unix_socket();
        match UnixStream::connect(path.clone()) {
            Ok(mut stream) => {
                match stream.write_all(UNIX_SOCKET_MSG) {
                    Err(cause) => Err(CheckUnixSocket{service: self.clone(), cause: cause}),
                    Ok(_) => {
                        // let mut response = String::new();
                        // stream.read_to_string(&mut response).unwrap();
                        Ok(format!("Service is listening on UNIX socket: {}", self.unix_socket()))
                    },
                }
            },
            Err(cause) => Err(CheckUnixSocketMissing{service: self.clone(), cause: cause}),
        }
    }


    fn checks_for(&self) -> Result<String, Mortal> {
        let mut checks_performed = 0;

        match self.name().as_ref() {
            "" =>
                return Err(CheckNameEmpty{service: self.clone()}),
            name =>
                trace!("Service name set: {}", name.underline()),
        }

        match self.unix_socket().as_ref() {
            "" => trace!("Undefined unix_socket for: {}", self.styled()),
            _  =>
                match self.try_unix_socket() {
                    Ok(_) => {
                        checks_performed += 1;
                        debug!("UNIX socket check passed for: {}, with unix_socket: {}", self.styled(), self.unix_socket())
                    },
                    Err(err) => return Err(err),
                },
        }

        match self.pid_file().as_ref() {
            "" => trace!("Undefined pid_file for: {}", self.styled()),
            _  =>
                match self.try_pid_file() {
                    Ok(_) => {
                        checks_performed += 1;
                        debug!("PID check passed for: {}, with pid_file: {}", self.styled(), self.pid_file())
                    },
                    Err(err) => return Err(err),
                },
        }

        if self.urls().len() > 0 {
            match self.try_urls() {
                Ok(_) => {
                    checks_performed += 1;
                    debug!("URLs check passed for: {}, with urls: {:?}", self.styled(), self.urls())
                },
                Err(err) => return Err(err),
            }
        } else {
            trace!("Undefined urls for: {}", self.styled())
        }


        trace!("performed {} checks for: {}", checks_performed, self.styled());
        let plu = match checks_performed {
            1 => "check",
            _ => "checks",
        };
        match checks_performed {
            0 => Err(CheckNoServiceChecks{service: self.clone()}),
            _ => Ok(format!("Ok ⇒ {} {} passed for: {}", format!("{:2}", checks_performed).bold(), plu, self.styled())),
        }
    }

}
