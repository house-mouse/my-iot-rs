//! [Telegram bot](https://core.telegram.org/bots/api) service able to receive and send messages.

use crate::prelude::*;
use log::debug;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json::json;
use std::fmt::Debug;
use std::time::Duration;

const CLIENT_TIMEOUT_SECS: u64 = 60;

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Telegram {
    pub secrets: Secrets,
}

/// Secrets section.
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Secrets {
    pub token: String,
}

impl Telegram {
    pub fn spawn(self, service_id: String, tx: Sender) -> Result<()> {
        let client = async_client_builder()
            .timeout(Duration::from_secs(CLIENT_TIMEOUT_SECS + 1))
            .build()?;

        tokio::spawn(async move {
            let mut offset: Option<i64> = None;
            loop {
                match self.iterate(&service_id, &client, offset, &tx).await {
                    Ok(new_offset) => offset = new_offset,
                    Err(error) => error!("[{}] Failed to fetch updates {}", service_id, error.to_string()),
                }
            }
        });

        Ok(())
    }

    async fn iterate(
        &self,
        service_id: &str,
        client: &Client,
        offset: Option<i64>,
        tx: &Sender,
    ) -> Result<Option<i64>> {
        let mut offset = offset;
        for update in self.get_updates(&client, offset).await?.iter() {
            offset = offset.max(Some(update.update_id + 1));
            self.send_readings(&service_id, &tx, &update);
        }
        debug!("{}: next offset: {:?}", &service_id, offset);
        Ok(offset)
    }

    /// Send reading messages from the provided Telegram update.
    fn send_readings(&self, service_id: &str, tx: &Sender, update: &TelegramUpdate) {
        debug!("{}: {:?}", service_id, &update);

        if let Some(ref message) = update.message {
            if let Some(ref text) = message.text {
                Message::new(format!("{}::{}::message", service_id, message.chat.id))
                    .type_(MessageType::ReadNonLogged)
                    .value(Value::Text(text.into()))
                    .timestamp(message.date)
                    .send_to(&tx);
            }
        }
    }

    /// <https://core.telegram.org/bots/api#getupdates>
    async fn get_updates(&self, client: &Client, offset: Option<i64>) -> Result<Vec<TelegramUpdate>> {
        self.call_api(
            client,
            "getUpdates",
            &json!({
                "offset": offset,
                "limit": null,
                "timeout": CLIENT_TIMEOUT_SECS,
                "allowed_updates": ["message"],
            }),
        )
        .await
    }

    /// Call [Telegram Bot API](https://core.telegram.org/bots/api) method.
    async fn call_api<P: Serialize + Debug + ?Sized, R: DeserializeOwned>(
        &self,
        client: &Client,
        method: &str,
        parameters: &P,
    ) -> Result<R> {
        debug!("{}({:?})", &method, parameters);
        // FIXME: https://github.com/eigenein/my-iot-rs/issues/44
        client
            .get(&format!(
                "https://api.telegram.org/bot{}/{}",
                &self.secrets.token, method
            ))
            .json(parameters)
            .send()
            .await?
            .json::<TelegramResponse<R>>()
            .await
            .map_err(Into::into)
            .and_then(|response| {
                if response.ok {
                    Ok(response.result.unwrap())
                } else {
                    error!("Telegram error: {:?}", response.description);
                    Err(InternalError::new(response.description.unwrap()).into())
                }
            })
    }
}

/// <https://core.telegram.org/bots/api#making-requests>
// TODO: I'd like rather to see it as an `enum`.
#[derive(Deserialize)]
pub struct TelegramResponse<T> {
    pub ok: bool,
    pub description: Option<String>,
    pub result: Option<T>,
}

#[derive(Deserialize, Debug)]
struct TelegramUpdate {
    pub update_id: i64,
    pub message: Option<TelegramMessage>,
}

/// <https://core.telegram.org/bots/api#message>
#[derive(Deserialize, Debug)]
pub struct TelegramMessage {
    pub message_id: i64,

    #[serde(deserialize_with = "chrono::serde::ts_seconds::deserialize")]
    pub date: DateTime<Utc>,

    pub chat: TelegramChat,
    pub text: Option<String>,
}

/// <https://core.telegram.org/bots/api#chat>
#[derive(Deserialize, Debug)]
pub struct TelegramChat {
    pub id: i64,
}

pub enum TelegramChatId {
    UniqueId(i64),
    Username(String),
}
