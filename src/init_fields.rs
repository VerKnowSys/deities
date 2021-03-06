use std::env;

use crate::{service::Service, *};


/// standard fields for service init file:
pub trait InitFields {
    fn disk_minimum_space(&self) -> i64;
    fn disk_minimum_inodes(&self) -> i64;
    fn slack_webhook_url(&self) -> String;
    fn slack_alert_channel(&self) -> String;
    fn checks_interval(&self) -> u64;
    fn checks_url_timeout(&self) -> u64;
    fn deathwatches_interval(&self) -> u64;
    fn user(&self) -> String;
    fn group(&self) -> String;
    fn work_dir(&self) -> String;
    fn pid_file(&self) -> String;
    fn unix_socket(&self) -> String;
    fn urls(&self) -> Vec<String>;
}


/// InitFields will try to use shell variables as fallback for most missin options in init files
impl InitFields for Service {
    /// minimum disk space on disk required - in MiB
    #[instrument]
    fn disk_minimum_space(&self) -> i64 {
        match self.disk_minimum_space {
            Some(disk_minimum_space) => disk_minimum_space,
            None => {
                match env::var("DISK_MINIMUM_SPACE") {
                    Ok(disk_minimum_space) => {
                        disk_minimum_space.parse().unwrap_or(DISK_MINIMUM_SPACE)
                    }
                    Err(_) => DISK_MINIMUM_SPACE,
                }
            }
        }
    }

    // minimum disk inodes on disk required
    #[instrument]
    fn disk_minimum_inodes(&self) -> i64 {
        match self.disk_minimum_inodes {
            Some(disk_minimum_inodes) => disk_minimum_inodes,
            None => {
                match env::var("DISK_MINIMUM_INODES") {
                    Ok(disk_minimum_inodes) => {
                        disk_minimum_inodes.parse().unwrap_or(DISK_MINIMUM_INODES)
                    }
                    Err(_) => DISK_MINIMUM_INODES,
                }
            }
        }
    }


    #[instrument]
    fn slack_webhook_url(&self) -> String {
        match self.slack_webhook_url.clone() {
            Some(slack_webhook_url) => slack_webhook_url,
            None => {
                match env::var("SLACK_WEBHOOK_URL") {
                    Ok(slack_webhook_url) => slack_webhook_url,
                    Err(_) => "".to_string(),
                }
            }
        }
    }

    #[instrument]
    fn slack_alert_channel(&self) -> String {
        match self.slack_alert_channel.clone() {
            Some(slack_alert_channel) => slack_alert_channel,
            None => {
                match env::var("SLACK_ALERT_CHANNEL") {
                    Ok(slack_alert_channel) => slack_alert_channel,
                    Err(_) => SLACK_ALERT_CHANNEL.to_string(),
                }
            }
        }
    }


    #[instrument]
    fn checks_interval(&self) -> u64 {
        match self.checks_interval {
            Some(checks_interval) => checks_interval,
            None => {
                match env::var("CHECKS_INTERVAL") {
                    Ok(checks_interval) => {
                        match checks_interval.parse().unwrap_or(CHECKS_INTERVAL) {
                            v if v < 100 => 100, /* pointless to do it more often than 10 times per second */
                            v => v,
                        }
                    }
                    Err(_) => CHECKS_INTERVAL,
                }
            }
        }
    }


    #[instrument]
    fn checks_url_timeout(&self) -> u64 {
        match self.checks_url_timeout {
            Some(checks_url_timeout) => checks_url_timeout,
            None => {
                match env::var("CHECKS_URL_TIMEOUT") {
                    Ok(timeout) => {
                        match timeout.parse().unwrap_or(CHECKS_URL_TIMEOUT) {
                            v if v < 1000 => 1000, /* pointless to expect url check to be less than a second */
                            v => v,
                        }
                    }
                    Err(_) => CHECKS_URL_TIMEOUT,
                }
            }
        }
    }


    #[instrument]
    fn deathwatches_interval(&self) -> u64 {
        match self.deathwatches_interval {
            Some(deathwatches_interval) => deathwatches_interval,
            None => {
                match env::var("DEATHWATCHES_INTERVAL") {
                    Ok(interval) => {
                        match interval.parse().unwrap_or(DEATHWATCHES_INTERVAL) {
                            v if v < 2000 => 2000, /* pointless to wait less than 2 seconds for process to react */
                            v => v,
                        }
                    }
                    Err(_) => DEATHWATCHES_INTERVAL,
                }
            }
        }
    }


    /// returns system name of user
    #[instrument]
    fn user(&self) -> String {
        match self.user.clone() {
            Some(name) => name,
            None => {
                match env::var("USER") {
                    Ok(somebody) => somebody,
                    Err(_) => "nobody".to_string(),
                }
            }
        }
    }


    /// returns system group name
    #[instrument]
    fn group(&self) -> String {
        match self.group.clone() {
            Some(group) => group,
            None => {
                match &self.sys_info().sysname[..] {
                    // on macOS it's better to assume that our user is in staff group:
                    "Darwin" => "staff".to_string(),
                    _ => "nobody".to_string(),
                }
            }
        }
    }


    /// returns service working dir
    #[instrument]
    fn work_dir(&self) -> String {
        match self.work_dir.clone() {
            Some(path) => path,
            None => "/tmp".to_string(),
        }
    }


    /// returns service pid file to monitor
    #[instrument]
    fn pid_file(&self) -> String {
        match self.pid_file.clone() {
            Some(path) => path,
            None => "".to_string(),
        }
    }


    /// returns path to unix socket to monitor
    #[instrument]
    fn unix_socket(&self) -> String {
        match self.unix_socket.clone() {
            Some(path) => path,
            None => "".to_string(),
        }
    }


    /// returns urls list to check
    #[instrument]
    fn urls(&self) -> Vec<String> {
        match self.urls.clone() {
            Some(vector) => vector,
            None => vec![],
        }
    }
}
