use slack_hook::{Slack, PayloadBuilder, AttachmentBuilder, Parse, Field}; // SlackLink,
use chrono::Local;
use chrono::datetime::*;
use slack_hook::SlackTextContent::{Text}; // Link
use hostname::get_hostname;
use uname::*;

use common::*;
use service::Service;


/*
 * Svarog is mr Smith - that can do variety of stuff
 */
pub trait Svarog {

    fn hostname(&self) -> String;

    fn notification(&self, message: String, error: String) -> Result<String, String>;

}


impl Svarog for Service {


    fn notification(&self, message: String, error: String) -> Result<String, String> {
        let os_handler: Info = uname().unwrap();
        let local: DateTime<Local> = Local::now();
        let slack = Slack::new(SLACK_WEBHOOK_URL).unwrap();
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
                            Field::new("Host name:", format!("{}", os_handler.nodename), Some(true)),
                            Field::new(
                                format!("System / Release / Machine / {}", NAME),
                                format!("{} / {} / {} / {}", os_handler.sysname, os_handler.release, os_handler.machine, VERSION),
                                Some(true)),

                            Field::new("", "", Some(true)),
                            Field::new("Error details:", error, Some(false)),
                        ])
                    .ts(&local.naive_local())
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
                Err(format!("Notification send failure: {:?}", cause)),
        }
    }


    /// Helper to read hostname from underlring system
    fn hostname(&self) -> String {
        match get_hostname() {
            Some(host) => host,
            None => DEFAULT_HOSTNAME.to_string(),
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
