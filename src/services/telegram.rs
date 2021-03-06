//! [Telegram bot](https://core.telegram.org/bots/api) service which is able to receive and send messages.

use std::fmt::Debug;
use std::time::Duration;

use bytes::Bytes;
use log::debug;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::prelude::*;
use crate::services::CLIENT;
use reqwest::blocking::multipart::{Form, Part};

const GET_UPDATES_TIMEOUT_SECS: u64 = 60;

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Telegram {
    secrets: Secrets,
}

/// Secrets section.
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Secrets {
    token: String,
}

impl Telegram {
    pub fn spawn(self, service_id: String, bus: &mut Bus) -> Result {
        let tx = bus.add_tx();

        thread::Builder::new().name(service_id.clone()).spawn(move || {
            let mut offset: Option<i64> = None;
            loop {
                match self.loop_(&service_id, offset, &tx) {
                    Ok(new_offset) => offset = new_offset,
                    Err(error) => {
                        error!("Failed to refresh the sensors: {}", error.to_string());
                        sleep(Duration::from_secs(60));
                    }
                }
            }
        })?;

        Ok(())
    }

    fn loop_(&self, service_id: &str, offset: Option<i64>, tx: &Sender) -> Result<Option<i64>> {
        let mut offset = offset;
        for update in self.get_updates(offset)?.iter() {
            offset = offset.max(Some(update.update_id + 1));
            self.send_readings(&service_id, &tx, &update)?;
        }
        debug!("{}: next offset: {:?}", &service_id, offset);
        Ok(offset)
    }

    /// Send reading messages from the provided Telegram update.
    fn send_readings(&self, service_id: &str, tx: &Sender, update: &TelegramUpdate) -> Result {
        debug!("{}: {:?}", service_id, &update);

        if let Some(ref message) = update.message {
            if let Some(ref text) = message.text {
                tx.send(
                    Message::new(format!("{}::{}::message", service_id, message.chat.id))
                        .type_(MessageType::ReadNonLogged)
                        .value(Value::Text(text.into()))
                        .timestamp(message.date),
                )?;
            }
        }

        Ok(())
    }
}

/// API.
impl Telegram {
    /// <https://core.telegram.org/bots/api#getupdates>
    fn get_updates(&self, offset: Option<i64>) -> Result<Vec<TelegramUpdate>> {
        self.call(
            &TelegramMethodCall::GetUpdates {
                offset,
                timeout: GET_UPDATES_TIMEOUT_SECS,
                allowed_updates: &["message"],
            },
            None,
        )
    }

    /// Calls a [Telegram Bot API](https://core.telegram.org/bots/api) method.
    pub fn call<R: DeserializeOwned>(
        &self,
        call: &TelegramMethodCall,
        input_file: Option<(String, Arc<Bytes>)>,
    ) -> Result<R> {
        debug!("{:?}", call);

        let url = format!(
            "https://api.telegram.org/bot{}/{}",
            self.secrets.token,
            match call {
                TelegramMethodCall::GetUpdates { .. } => "getUpdates",
                TelegramMethodCall::SendMessage { .. } => "sendMessage",
                TelegramMethodCall::SendVideo { .. } => "sendVideo",
            },
        );

        let mut request = match input_file {
            Some((field_name, bytes)) => CLIENT
                .post(&url)
                .query(call)
                .multipart(Form::new().part(field_name, Part::bytes(bytes.to_vec()).file_name(""))),
            None => CLIENT.get(&url).json(call),
        };

        // `GetUpdates` requires a timeout that is at least as long as the one in the request itself.
        if let TelegramMethodCall::GetUpdates { .. } = call {
            request = request.timeout(Duration::from_secs(GET_UPDATES_TIMEOUT_SECS + 1));
        }

        match request.send()?.json::<TelegramResponse<R>>()? {
            TelegramResponse::Result { result } => Ok(result),
            TelegramResponse::Error { description } => {
                error!("Telegram error: {:?}", description);
                Err(description.into())
            }
        }
    }
}

/// <https://core.telegram.org/bots/api#making-requests>
#[derive(Deserialize)]
#[serde(untagged)]
pub enum TelegramResponse<T> {
    Result { result: T },
    Error { description: String },
}

#[derive(Deserialize, Debug)]
pub struct TelegramUpdate {
    pub update_id: i64,
    pub message: Option<TelegramMessage>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum TelegramChatId {
    UniqueId(i64),

    #[allow(unused)]
    Username(String),
}

/// <https://core.telegram.org/bots/api#message>
#[derive(Deserialize, Debug, Clone)]
pub struct TelegramMessage {
    pub message_id: i64,

    #[serde(deserialize_with = "chrono::serde::ts_seconds::deserialize")]
    pub date: DateTime<Utc>,

    pub chat: TelegramChat,
    pub text: Option<String>,
}

/// <https://core.telegram.org/bots/api#chat>
#[derive(Deserialize, Debug, Clone)]
pub struct TelegramChat {
    pub id: i64,
}

#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum TelegramMethodCall {
    /// https://core.telegram.org/bots/api#getupdates
    GetUpdates {
        #[serde(skip_serializing_if = "Option::is_none")]
        offset: Option<i64>,

        timeout: u64,

        allowed_updates: &'static [&'static str],
    },

    /// <https://core.telegram.org/bots/api#sendmessage>
    SendMessage {
        chat_id: TelegramChatId,
        text: String,

        /// <https://core.telegram.org/bots/api#formatting-options>
        #[serde(skip_serializing_if = "Option::is_none")]
        parse_mode: Option<String>,
    },

    /// <https://core.telegram.org/bots/api#sendvideo>
    SendVideo {
        chat_id: TelegramChatId,

        /// Only allows to pass a URL or a file ID.
        /// Use `input_file` parameter to send a `Bytes`.
        #[serde(skip_serializing_if = "Option::is_none")]
        video: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        caption: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        parse_mode: Option<String>,
    },
}
