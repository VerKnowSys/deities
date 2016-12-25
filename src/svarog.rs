use slack_hook::{Slack, PayloadBuilder, AttachmentBuilder, Parse};
use chrono::Local;
use chrono::datetime::*;
use slack_hook::SlackTextContent::{Text};

use common::*;


/*
 * Svarog is mr Smith - that can do variety of stuff
 */
pub struct Svarog;


impl Svarog {


    pub fn notification(message: String) -> Result<String, String> {
        let local: DateTime<Local> = Local::now();
        let slack = Slack::new(SLACK_WEBHOOK_URL).unwrap();
        let p = PayloadBuilder::new()
            .attachments(
                vec![
                    AttachmentBuilder::new("Failure")
                    .title("Problem notification")
                    .color("#FF3d41")
                    .text(
                        vec![
                          Text(message.into()),
                          // Link(SlackLink::new("https://google.com", "Google")),
                          // Text("".into())
                        ].as_slice())
                    .ts(&local.naive_local())
                    .build()
                    .unwrap()
                ])
            .link_names(true)
            .unfurl_links(true)
            .unfurl_media(true)
            .username("Svarog")
            .icon_emoji(":rotating_light:")
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
