use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use curl::easy::Easy;
use std::time::Duration;
use std::path::Path;
use libc::kill;

use common::*;
use service::Service;
use svarog::Svarog;
use mortal::Mortal;
use mortal::Mortal::*;


/*
 * Perun is a supervisor deity
 */
pub trait Perun {

    fn try_pid_file(&self) -> Result<Mortal, Mortal>;
    fn try_unix_socket(&self) -> Result<Mortal, Mortal>;
    fn try_urls(&self) -> Result<Mortal, Mortal>;

    fn checks_for(&self) -> Result<Mortal, Mortal>;
}


impl Perun for Service {


    fn try_urls(&self) -> Result<Mortal, Mortal> {
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
        Ok(OkUrlsChecks{service: self.clone()})
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


    fn try_unix_socket(&self) -> Result<Mortal, Mortal> {
        let path = self.clone().unix_socket();
        match UnixStream::connect(path.clone()) {
            Ok(mut stream) => {
                match stream.write_all(UNIX_SOCKET_MSG) {
                    Err(cause) => Err(CheckUnixSocket{service: self.clone(), cause: cause}),
                    Ok(_) => {
                        // let mut response = String::new();
                        // stream.read_to_string(&mut response).unwrap();
                        Ok(OkUnixSockCheck{service: self.clone()})
                    },
                }
            },
            Err(cause) => Err(CheckUnixSocketMissing{service: self.clone(), cause: cause}),
        }
    }


    fn checks_for(&self) -> Result<Mortal, Mortal> {
        let mut checks_performed = 0;

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
        match checks_performed {
            0 => Err(CheckNoServiceChecks{service: self.clone()}),
            _ => Ok(OkAllChecks{service: self.clone(), amount: checks_performed}),
        }
    }

}
