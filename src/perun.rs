use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use libc::*;
use colored::*;
use curl::easy::Easy;
use std::time::Duration;
use std::path::Path;
use std::io::{Error, ErrorKind};

use common::*;
use mortal::Mortal;
use mortal::Mortal::*;
use service::Service;


/*
 * Perun is a supervisor deity
 */
pub trait Perun {
    fn try_pid_file(&self) -> Result<String, Mortal>;
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


    fn try_pid_file(&self) -> Result<String, Mortal> {
        let path = self.clone().pid_file();
        match Service::load_raw(path.clone()) {
            Ok(raw_content) => {
                let content = raw_content.trim();
                match content.parse::<i32>() {
                    Ok(pid) => unsafe {
                        match kill(pid, 0) {
                            0 => Ok(format!("PID: {}, from file: {}, is alive!", pid, self.pid_file())),
                            _ => Err(CheckPidDead{service: self.clone(), pid: pid}),
                        }
                    },
                    Err(cause) => Err(CheckPidfileMalformed{service: self.clone(), cause: cause}),
                }
            },
            Err(cause) => Err(CheckPidfileUnaccessible{service: self.clone(), cause: Error::new(ErrorKind::PermissionDenied, cause.to_string())}),
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
