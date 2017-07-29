use slack_hook::{Slack, PayloadBuilder, AttachmentBuilder, Parse, Field}; // SlackLink,
use chrono::Local;
use chrono::datetime::*;
use slack_hook::SlackTextContent::{Text}; // Link
use hostname::get_hostname;
use uname::{uname, Info};
use std::io::{Error, ErrorKind};
use std::thread::sleep;
use std::time::Duration;
use libc;
use libc::kill;

use common::*;
use service::Service;
use mortal::Mortal;
use mortal::Mortal::*;


/*
 * Svarog is mr Smith - that can do variety of stuff
 */
pub trait Svarog {

    /// returns current hostname
    fn hostname(&self) -> String;


    /// access to general system info
    fn sys_info(&self) -> Info;


    /// sends Slack alert notifications
    fn notification(&self, message: String, error: String, webhookurl: String) -> Result<String, Mortal>;


    /// death_watch will kill service gracefully in case of failure
    /// instead of killing forcefully (kill -9)
    fn death_watch(&self, signal: libc::c_int) -> Result<Mortal, Mortal>;

    /// read pid from service pid file
    fn read_pid(&self) -> Result<i32, Mortal>;

    /// returns raw value of pid of service process
    fn pid(&self) -> i32;

}


impl Svarog for Service {


    fn notification(&self, message: String, error: String, webhookurl: String) -> Result<String, Mortal> {
        let local: DateTime<Local> = Local::now();
        let slack = Slack::new(webhookurl.as_ref()).unwrap();
        let p = PayloadBuilder::new()
            .attachments(
                vec![
                    AttachmentBuilder::new(DEFAULT_NOTIFICATION_NAME)
                    .title("ALERT NOTIFICATION")
                    .author_name(DEFAULT_NOTIFICATION_NAME)
                    .author_icon(DEFAULT_VKS_LOGO)
                    .color("#FF3d41")
                    .text(
                        vec![
                            Text("Unstable service detected. Deities will attempt to solve this problem automatically.".into()),
                            // Link(SlackLink::new("https://google.com", "Google")),
                            Text("".into()),
                        ].as_slice())
                    .fields(
                        vec![
                            Field::new("", "", Some(false)),
                            Field::new("", "", Some(false)),
                            Field::new("Message:", message, Some(true)),
                            Field::new("Service details:", self.to_string(), Some(true)),
                            Field::new("", "", Some(false)),
                            Field::new("Host name:", format!("{}", self.sys_info().nodename), Some(true)),
                            Field::new(
                                format!("System / Release / Machine / {}", NAME),
                                format!("{} / {} / {} / {}", self.sys_info().sysname, self.sys_info().release, self.sys_info().machine, VERSION),
                                Some(true)),

                            Field::new("", "", Some(true)),
                            Field::new("Error details:", error, Some(false)),
                        ])
                    // .ts(&local.naive_local())
                    .footer_icon(DEFAULT_VKS_LOGO)
                    .footer(vec![
                        Text("© 2o16-2o17   |".into()),
                        ].as_slice())
                    .build()
                    .unwrap()
                ])
            .link_names(true)
            .unfurl_links(true)
            .unfurl_media(true)
            .username(DEFAULT_NOTIFICATION_NAME)
            .icon_url(DEFAULT_VKS_LOGO)
            .icon_emoji(":rotating_light:")
            .text("")
            .channel(SLACK_ALERT_CHANNEL)
            .parse(Parse::Full)
            .build()
            .unwrap();

        let res = slack.send(&p);
        match res {
            Ok(()) =>
                Ok("Notifiication sent".to_string()),
            Err(cause) =>
                Err(NotificationFailure{cause: cause}),
        }
    }


    /// Helper to read hostname from underlring system
    fn hostname(&self) -> String {
        match get_hostname() {
            Some(host) => host,
            None => DEFAULT_HOSTNAME.to_string(),
        }
    }


    // helper to read basic system information
    fn sys_info(&self) -> Info {
        uname().unwrap_or(Info::new().unwrap())
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


    // fn parse() -> Option<Perun> {
    //     match Svarog::load_file() {
    //         Some(content) => {
    //             let mut parser = toml::Parser::new(content.as_ref());
    //             match parser.parse() {
    //                 Some(toml) => {
    //                     toml.lookup("")
    //                 },
    //                 None => {
    //                     for err in &parser.errors {
    //                         let (loline, locol) = parser.to_linecol(err.lo);
    //                         let (hiline, hicol) = parser.to_linecol(err.hi);
    //                         println!("{}:{}:{}-{}:{} error: {}",
    //                                  self.name, loline, locol, hiline, hicol, err.desc);
    //                     }
    //                     panic!("Parsing definition failed!")
    //                 }
    //             };
    //             None
    //         },
    //         None => {
    //             println!("Nothing to parse.");
    //             None
    //         }
    //     }
    // }


}
