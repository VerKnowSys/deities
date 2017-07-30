use std::env;

use common::*;
use svarog::Svarog;
use service::Service;


/// standard fields for service init file:
pub trait InitFields {
    fn disk_space(&self) -> i64;
    fn disk_inodes(&self) -> i64;
    fn slack_webhookurl(&self) -> String;
    fn slack_alertchannel(&self) -> String;
    fn check_interval(&self) -> u64;
    fn check_urltimeout(&self) -> u64;
    fn deathwatch_interval(&self) -> u64;
    fn user(&self) -> String;
    fn group(&self) -> String;
    fn work_dir(&self) -> String;
    fn pid_file(&self) -> String;
    fn unix_socket(&self) -> String;
    fn urls(&self) -> Vec<String>;
    fn ini_file(&self) -> String;
}


/// InitFields will try to use shell variables as fallback for most missin options in init files
impl InitFields for Service {


    /// minimum disk space on disk required - in MiB
    fn disk_space(&self) -> i64 {
        match self.disk_space.clone() {
            Some(disk_space) => disk_space,
            None =>
                match env::var("DISK_MINIMUMSPACE") {
                    Ok(disk_space) => disk_space.parse().unwrap_or(DISK_MINIMUMSPACE),
                    Err(_) => DISK_MINIMUMSPACE,
                },
        }
    }

    // minimum disk inodes on disk required
    fn disk_inodes(&self) -> i64 {
        match self.disk_inodes.clone() {
            Some(disk_inodes) => disk_inodes,
            None =>
                match env::var("DISK_MINIMUMINODES") {
                    Ok(disk_inodes) => disk_inodes.parse().unwrap_or(DISK_MINIMUMINODES),
                    Err(_) => DISK_MINIMUMINODES,
                },
        }
    }


    fn slack_webhookurl(&self) -> String {
        match self.slack_webhookurl.clone() {
            Some(slack_webhookurl) => slack_webhookurl,
            None =>
                match env::var("SLACK_WEBHOOKURL") {
                    Ok(slack_webhookurl) => slack_webhookurl.to_owned(),
                    Err(_) => "".to_string(),
                },
        }
    }


    fn slack_alertchannel(&self) -> String {
        match self.slack_alertchannel.clone() {
            Some(slack_alertchannel) => slack_alertchannel,
            None =>
                match env::var("SLACK_ALERTCHANNEL") {
                    Ok(slack_alertchannel) => slack_alertchannel.to_owned(),
                    Err(_) => SLACK_ALERT_CHANNEL.to_string(),
                },
        }
    }


    fn check_interval(&self) -> u64 {
        match self.check_interval.clone() {
            Some(check_interval) => check_interval,
            None =>
                match env::var("CHECK_INTERVAL") {
                    Ok(check_interval) =>
                        match check_interval.parse().unwrap_or(CHECK_INTERVAL) {
                            v if v < 100 => 100, // pointless to do it more often than 10 times per second
                            v => v
                        },
                    Err(_) => CHECK_INTERVAL,
                },
        }
    }


    fn check_urltimeout(&self) -> u64 {
        match self.check_urltimeout.clone() {
            Some(check_urltimeout) => check_urltimeout,
            None =>
                match env::var("CHECK_URL_TIMEOUT") {
                    Ok(timeout) =>
                        match timeout.parse().unwrap_or(CHECK_URL_TIMEOUT) {
                            v if v < 1000 => 1000, // pointless to expect url check to be less than a second
                            v => v
                        },
                    Err(_) => CHECK_URL_TIMEOUT,
                },
        }
    }


    fn deathwatch_interval(&self) -> u64 {
        match self.deathwatch_interval.clone() {
            Some(deathwatch_interval) => deathwatch_interval,
            None =>
                match env::var("DEATHWATCH_INTERVAL") {
                    Ok(interval) =>
                        match interval.parse().unwrap_or(DEATHWATCH_INTERVAL) {
                            v if v < 2000 => 2000, // pointless to wait less than 2 seconds for process to react
                            v => v
                        },
                    Err(_) => DEATHWATCH_INTERVAL,
                },
        }
    }


    /// returns system name of user
    fn user(&self) -> String {
        match self.user.clone() {
            Some(name) => name,
            None =>
                match env::var("USER") {
                    Ok(somebody) => somebody.to_owned(),
                    Err(_) => "nobody".to_string(),
                },
        }
    }


    /// returns system group name
    fn group(&self) -> String {
        match self.group.clone() {
            Some(group) => group,
            None =>
                match &self.sys_info().sysname[..] {
                    // on macOS it's better to assume that our user is in staff group:
                    "Darwin" => "staff".to_string(),
                    _ => "nobody".to_string(),
                },
        }
    }


    /// returns service working dir
    fn work_dir(&self) -> String {
        match self.work_dir.clone() {
            Some(path) => path,
            None => "/tmp".to_string(),
        }
    }


    /// returns service pid file to monitor
    fn pid_file(&self) -> String {
        match self.pid_file.clone() {
            Some(path) => path,
            None => "".to_string(),
        }
    }


    /// returns path to unix socket to monitor
    fn unix_socket(&self) -> String {
        match self.unix_socket.clone() {
            Some(path) => path,
            None => "".to_string(),
        }
    }


    /// returns urls list to check
    fn urls(&self) -> Vec<String> {
        match self.urls.clone() {
            Some(vector) => vector,
            None => vec!(),
        }
    }


    /// returns init file name
    fn ini_file(&self) -> String {
        match self.ini_file.clone() {
            Some(file_path) => file_path,
            None => "undefined-ini-file".to_string(),
        }
    }


}
