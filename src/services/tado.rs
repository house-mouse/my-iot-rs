//! [tado°](https://www.tado.com/) API.

use crate::prelude::*;
use reqwest::Client;
use reqwest::Url;
use std::time::{Duration, SystemTime};

const CLIENT_ID: &str = "public-api-preview";
const CLIENT_SECRET: &str = "4HJGRffVR8xb3XdEUQpjgZ1VplJi6Xgw";
const SCOPE: &str = "home.user";
const REFRESH_PERIOD: Duration = Duration::from_millis(60000);

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Tado {
    pub secrets: Secrets,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Secrets {
    pub email: String,
    pub password: String,
}

impl Tado {
    pub fn spawn(self, _service_id: String, _tx: Sender) -> Result<()> {
        let _client = async_client_builder().build()?;

        tokio::spawn(async move {
            loop {
                tokio::time::delay_for(REFRESH_PERIOD).await;
            }
        });

        Ok(())
    }

    #[allow(dead_code)]
    async fn login(&self, client: &Client) -> Result<LoginResponse> {
        debug!("Logging in…");
        let response = client
            .post(Url::parse_with_params(
                "https://auth.tado.com/oauth/token",
                &[
                    ("client_id", CLIENT_ID),
                    ("client_secret", CLIENT_SECRET),
                    ("grant_type", "password"),
                    ("scope", SCOPE),
                    ("username", &self.secrets.email),
                    ("password", &self.secrets.password),
                ],
            )?)
            .send()
            .await?
            .json::<LoginResponse>()
            .await?;
        debug!("Logged in, token expires at: {:?}", response.expires_at);
        Ok(response)
    }
}

#[derive(Deserialize)]
struct LoginResponse {
    pub access_token: String,

    #[serde(rename = "expires_in", deserialize_with = "deserialize_expires_at")]
    pub expires_at: SystemTime,

    pub refresh_token: String,
}

fn deserialize_expires_at<'de, D: Deserializer<'de>>(deserializer: D) -> std::result::Result<SystemTime, D::Error> {
    Ok(SystemTime::now() + Duration::from_secs(Deserialize::deserialize(deserializer)?))
}
