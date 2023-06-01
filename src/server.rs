use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use anyhow::{Error, Result};
// FIXME: should be in lib
use himalaya::config::DeserializedConfig;
#[allow(unused_imports)]
use log::{error, info, warn};
use pimalaya_email::BackendBuilder;
pub use pimalaya_email::{Envelope, Envelopes, Flag};

pub type MailId = String;
pub type AccountId = String;
pub type MboxName = String;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct MboxId {
    pub account: AccountId,
    pub name: MboxName,
}

pub type Mboxes = HashMap<AccountId, Vec<MboxName>>;

pub struct Server {
    pub to: Sender<ServerCmd>,
    pub from: Receiver<ServerEvent>,
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum ServerCmd {
    GetAllEnvelopes(MboxName),
    GetMessageBody(MailId, MboxName),
    GetMboxes,
}

#[derive(Debug)]
pub enum ServerEvent {
    Envelopes((MboxId, Envelopes)),
    Mboxes(Vec<MboxName>),
    Body((MailId, String)),
    Error(Error),
}

pub fn accounts() -> Result<Vec<String>> {
    let config = DeserializedConfig::from_opt_path(None)?;
    let accounts = config.accounts.keys().cloned().collect();
    Ok(accounts)
}

pub fn run(ctx: egui::Context, account: String) -> (Sender<ServerCmd>, Receiver<ServerEvent>) {
    let (to_main, from_server) = channel();
    let (to_server, from_main) = channel();

    thread::spawn(move || {
        info!("Server thread spawn");

        let config = DeserializedConfig::from_opt_path(None).unwrap();
        let account_config = config.to_account_config(Some(&account)).unwrap();
        let backend = BackendBuilder::new().build(&account_config).unwrap();
        let mut db = HashMap::new();

        loop {
            let message = from_main.recv();

            info!("Got server command: {:?}", message);

            let message = match message {
                Ok(m) => m,
                Err(e) => {
                    to_main
                        .send(ServerEvent::Error(e.into()))
                        .expect("Main thread dead?");
                    continue;
                }
            };

            let main_message = match message {
                // FIXME: should use 0 instead of 1024, but getting parsing error
                ServerCmd::GetAllEnvelopes(folder) => {
                    match backend.list_envelopes(&folder, 1024, 0) {
                        Err(e) => to_main.send(ServerEvent::Error(e.into())),
                        Ok(envelopes) => {
                            let envelopes_as_map: HashMap<String, Envelope> = envelopes
                                .to_vec()
                                .into_iter()
                                .map(|i| (i.id.clone(), i))
                                .collect();
                            db.insert(folder.clone(), envelopes_as_map);
                            let mbox = MboxId {
                                account: account.clone(),
                                name: folder,
                            };
                            to_main.send(ServerEvent::Envelopes((mbox, envelopes)))
                        }
                    }
                }
                ServerCmd::GetMessageBody(id, folder) => {
                    match backend.get_emails(&folder, vec![&id]) {
                        Err(e) => to_main.send(ServerEvent::Error(e.into())),
                        Ok(msg) => {
                            let tpl = msg
                                .first()
                                .unwrap()
                                .to_read_tpl(&account_config, |i| i)
                                .unwrap();
                            to_main.send(ServerEvent::Body((id, tpl.into())))
                        }
                    }
                }
                ServerCmd::GetMboxes => match backend.list_folders() {
                    Err(e) => to_main.send(ServerEvent::Error(e.into())),
                    Ok(folders) => {
                        let mboxes: Vec<MboxName> = folders
                            .iter()
                            .filter(|f| f.name != "[Gmail]")
                            .map(|folder| folder.name.clone())
                            .collect();
                        to_main.send(ServerEvent::Mboxes(mboxes))
                    }
                },
            };

            if let Err(e) = main_message {
                error!("Communitcation with `main` failed: {}", e);
            }

            ctx.request_repaint();
        }
    });
    (to_server, from_server)
}
