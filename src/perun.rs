use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use curl::easy::Easy;
use std::time::Duration;
use std::path::Path;
use libc::kill;
use std::process::Command;
use regex::Regex;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use common::*;
use service::Service;
use svarog::Svarog;
use init_fields::InitFields;
use mortal::Mortal;
use mortal::Mortal::*;


/*
 * Perun is a supervisor deity
 */
pub trait Perun {

    fn try_pid_file(&self) -> Result<Mortal, Mortal>;
    fn try_unix_socket(&self) -> Result<Mortal, Mortal>;
    fn try_urls(&self) -> Result<Mortal, Mortal>;
    fn try_disk_check(&self) -> Result<Mortal, Mortal>;

    fn checks_for(&self) -> Result<Mortal, Mortal>;
    fn check_disk_space(&self) -> (i64, i64);
}


impl Perun for Service {


    fn try_urls(&self) -> Result<Mortal, Mortal> {
        for url in self.urls() {
            // let mut dst = Vec::new();
            let mut easy = Easy::new();
            easy.connect_timeout(Duration::from_millis(self.clone().check_urltimeout())).unwrap();
            easy.timeout(Duration::from_millis(self.clone().check_urltimeout())).unwrap();
            easy.dns_cache_timeout(Duration::from_millis(self.clone().check_urltimeout())).unwrap();
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


    fn try_disk_check(&self) -> Result<Mortal, Mortal> {
        match self.check_disk_space() {
            (space, _) if space / 1024 < DISK_MINIMUMSPACE => Err(CheckDiskSpace {service: self.clone()}),
            (_, inodes) if inodes < DISK_MINIMUMINODES => Err(CheckDiskInodes {service: self.clone()}),
            (_space, _inodes) => Ok(OkDiskCheck {service: self.clone()}),
        }
    }


    /*

    Linux => (Inodes check was not implemented due to unability to perform such check with single df command under GNU base)

        "df /"
        #
        # Filesystem     1K-blocks    Used Available Use% Mounted on
        # /dev/vda1       20510568 3157596  16297100  17% /


        "df -i /"
        # Filesystem      Inodes  IUsed   IFree IUse% Mounted on
        # /dev/vda1      1305600 134125 1171475   11% /


    *BSD | Darwin =>

        "df -i /"
        # HardenedBSD:
        #
        # Filesystem         1K-blocks   Used    Avail Capacity iused    ifree %iused  Mounted on
        # zroot/ROOT/default  49815548 817556 48997992     2%   49351 97995984    0%   /

        "df -i /"
        # Darwin:
        #
        # Filesystem   Size   Used  Avail Capacity iused      ifree %iused  Mounted on
        # /dev/disk1  233Gi   70Gi  162Gi    31% 1545816 4293421463    0%   /

    */

    #[cfg(all(target_os="linux"))]
    fn check_disk_space(&self) -> (i64, i64) {
        lazy_static! {
            static ref FIRST_LINE: Regex = Regex::new(r"^.*\n").unwrap();
            static ref SPACE: Regex = Regex::new(r"(\s+)").unwrap();
        }
        match Command::new("/bin/df").arg("-k").arg("/").output() {
            Ok(data) => {
                match String::from_utf8(data.stdout) {
                    Ok(parsed) => {
                        let inodes_data = FIRST_LINE.replace(parsed.as_ref(), "");
                        let mut it = SPACE.split(inodes_data.as_ref());
                        it.next(); it.next();
                        let free_disk_space_bytes = match it.next() {
                            Some(content) =>
                                match content.parse() {
                                    Ok(number) => number,
                                    Err(cause) => {
                                        error!("Parse failure. Reason: {:?}", cause);
                                        -1
                                    },
                                },
                            None => 0,
                        };

                        debug!("Free disk space: {} MiB. (inodes: Skipped for Linux)", free_disk_space_bytes / 1024);
                        (free_disk_space_bytes, 1000000)
                    },
                    Err(cause) => {
                        error!("Failed utf8 parse! Reason: {:?}", cause);
                        (-2, -2)
                    }
                }
            },
            Err(cause) => {
                error!("Failure! Reason: {:?}", cause);
                (-1, -1)
            }
        }
    }


    #[cfg(not(target_os="linux"))]
    fn check_disk_space(&self) -> (i64, i64) {
        lazy_static! {
            static ref FIRST_LINE: Regex = Regex::new(r"^.*\n").unwrap();
            static ref SPACE: Regex = Regex::new(r"(\s+)").unwrap();
        }
        match Command::new("/bin/df").arg("-ki").arg("/").output() {
            Ok(data) => {
                match String::from_utf8(data.stdout) {
                    Ok(parsed) => {
                        let inodes_data = FIRST_LINE.replace(parsed.as_ref(), "");
                        let mut it = SPACE.split(inodes_data.as_ref());
                        it.next(); it.next(); it.next();
                        let free_disk_space_bytes = match it.next() {
                            Some(content) =>
                                match content.parse() {
                                    Ok(number) => number,
                                    Err(cause) => {
                                        error!("Parse failure. Reason: {:?}", cause);
                                        -1
                                    },
                                },
                            None => 0,
                        };
                        it.next(); it.next();
                        let free_disk_inodes = match it.next() {
                            Some(content) =>
                                match content.parse() {
                                    Ok(number) => number,
                                    Err(cause) => {
                                        error!("Parse failure. Reason: {:?}", cause);
                                        -1
                                    },
                                },
                            None => 0,
                        };
                        debug!("Free disk space: {} GiB. Free inodes: {}", free_disk_space_bytes / 1024 / 1024, free_disk_inodes);
                        (free_disk_space_bytes, free_disk_inodes)
                    },
                    Err(cause) => {
                        error!("Failed utf8 parse! Reason: {:?}", cause);
                        (-2, -2)
                    }
                }
            },
            Err(cause) => {
                error!("Failure! Reason: {:?}", cause);
                (-1, -1)
            }
        }
    }


    fn checks_for(&self) -> Result<Mortal, Mortal> {
        let checks_performed = Arc::new(AtomicUsize::new(0));

        match self.try_disk_check() {
            Ok(_) => {
                checks_performed.fetch_add(1, Ordering::SeqCst);
                debug!("Disk checks passed for: {}", self.styled())
            },
            Err(cause) => return Err(cause),
        }


        match self.unix_socket().as_ref() {
            "" => trace!("Undefined unix_socket for: {}", self.styled()),
            _  =>
                match self.try_unix_socket() {
                    Ok(_) => {
                        checks_performed.fetch_add(1, Ordering::SeqCst);
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
                        checks_performed.fetch_add(1, Ordering::SeqCst);
                        debug!("PID check passed for: {}, with pid_file: {}", self.styled(), self.pid_file())
                    },
                    Err(err) => return Err(err),
                },
        }

        if self.urls().len() > 0 {
            match self.try_urls() {
                Ok(_) => {
                    checks_performed.fetch_add(1, Ordering::SeqCst);
                    debug!("URLs check passed for: {}, with urls: {:?}", self.styled(), self.urls())
                },
                Err(err) => return Err(err),
            }
        } else {
            trace!("Undefined urls for: {}", self.styled())
        }

        trace!("performed {} checks for: {}", checks_performed.clone().load(Ordering::SeqCst), self.styled());
        match checks_performed.clone().load(Ordering::SeqCst) {
            0 => Err(CheckNoServiceChecks{service: self.clone()}),
            _ => Ok(OkAllChecks{service: self.clone(), amount: checks_performed.clone().load(Ordering::SeqCst) as i32}),
        }
    }

}
