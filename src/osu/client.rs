use crate::osu::models::{
    CreatePmRequestBody, CreatePmResponseBody, RefreshAccessTokenRequestBody,
    RefreshAccessTokenResponseBody, SendMessageToChannelRequestBody,
    SendMessageToChannelResponseBody,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::time::{Duration, SystemTime};

const CACHE_PATH: &str = "./cache.json";

#[derive(Serialize, Deserialize, Debug)]
struct AccessToken {
    pub token: String,
    pub live_until: SystemTime,
}

#[derive(Serialize, Deserialize, Debug)]
struct OsuCache {
    pub access_token: Option<AccessToken>,
    pub refresh_token: String,
    pub player_id: u64,
    pub channel_id: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OsuConfig {
    player_id: u64,
    client_id: u64,
    client_secret: String,
    initial_refresh_token: String,
}

pub struct OsuClient {
    client: reqwest::Client,

    client_id: u64,
    client_secret: String,

    cache: OsuCache,
}

impl OsuClient {
    pub async fn create_async(config: OsuConfig) -> OsuClient {
        let mut cache: OsuCache;

        if let Ok(true) = fs::exists(CACHE_PATH) {
            let content = tokio::fs::read_to_string(CACHE_PATH).await.unwrap();

            cache = serde_json::from_str(&content).unwrap();

            if cache.player_id != config.player_id {
                cache.channel_id = None;
                cache.player_id = config.player_id;
            }
        } else {
            cache = OsuCache {
                player_id: config.player_id,
                refresh_token: config.initial_refresh_token,
                channel_id: None,
                access_token: None,
            };
        }

        OsuClient {
            client: reqwest::Client::new(),
            client_id: config.client_id,
            client_secret: config.client_secret,
            cache,
        }
    }

    pub async fn execute_send_message_async(
        &mut self,
        message: String,
    ) -> Result<(), Box<dyn Error>> {
        if self.check_update_access_token_async().await? {
            self.save_cache_async().await?;
        }

        if self.cache.channel_id.is_none() {
            let channel_id = self.create_new_pm_async(message).await?;
            self.cache.channel_id = Some(channel_id);
            self.save_cache_async().await?;
        } else {
            self.send_message_async(message).await?;
        }

        Ok(())
    }

    /// true если обновил
    async fn check_update_access_token_async(&mut self) -> Result<bool, Box<dyn Error>> {
        let now = SystemTime::now();

        let mut update_access_token = true;

        if let Some(access_token) = &self.cache.access_token {
            update_access_token = access_token.live_until <= (now + Duration::from_secs(60));
        }

        if !update_access_token {
            return Ok(false);
        }

        let response = OsuClient::update_access_token_async(self).await?;

        let time = SystemTime::now();

        self.cache.access_token = Some(AccessToken {
            token: response.access_token,
            live_until: time + Duration::from_secs(response.expires_in),
        });
        self.cache.refresh_token = response.refresh_token;

        Ok(true)
    }

    async fn update_access_token_async(
        &self,
    ) -> Result<RefreshAccessTokenResponseBody, Box<dyn Error>> {
        let body = RefreshAccessTokenRequestBody {
            client_id: self.client_id,
            client_secret: self.client_secret.clone(),
            refresh_token: self.cache.refresh_token.clone(),
            grant_type: "refresh_token".to_string(),
        };

        let response: RefreshAccessTokenResponseBody = self
            .post_async("https://osu.ppy.sh/oauth/token", &body, false)
            .await?;

        Ok(response)
    }

    async fn create_new_pm_async(&mut self, message: String) -> Result<u64, Box<dyn Error>> {
        let body = CreatePmRequestBody {
            target_id: self.cache.player_id,
            is_action: false,
            message,
            uuid: None,
        };

        let response: CreatePmResponseBody = self
            .post_async("https://osu.ppy.sh/api/v2/chat/new", &body, true)
            .await?;

        self.cache.channel_id = Some(response.channel.channel_id);

        Ok(response.channel.channel_id)
    }

    async fn send_message_async(&self, message: String) -> Result<(), Box<dyn Error>> {
        let Some(channel_id) = self.cache.channel_id else {
            return Err("self.target_channel_id is not set".into());
        };

        let body = SendMessageToChannelRequestBody {
            message,
            is_action: false,
        };

        let _: SendMessageToChannelResponseBody = self
            .post_async(
                format!("https://osu.ppy.sh/api/v2/chat/channels/{channel_id}/messages"),
                &body,
                true,
            )
            .await?;

        Ok(())
    }

    async fn post_async<TRequest, TResponse, U>(
        &self,
        url: U,
        body: &TRequest,
        auth: bool,
    ) -> Result<TResponse, Box<dyn Error>>
    where
        TRequest: ?Sized + Serialize,
        TResponse: ?Sized + DeserializeOwned,
        U: reqwest::IntoUrl,
    {
        let body = serde_json::to_string(&body).unwrap();

        // println!("{body}");

        let mut builder = self
            .client
            .post(url)
            .body(body)
            .header("Content-Type", "application/json");

        if auth {
            let Some(access_token) = &self.cache.access_token else {
                return Err("post_async auth is none".into());
            };

            builder = builder.header("Authorization", format!("Bearer {}", access_token.token));
        }

        let response = builder.send().await?;

        let response = response.error_for_status()?;

        let response = response.text().await?;

        // println!("{response}");

        let response = serde_json::from_str::<TResponse>(&response)?;

        Ok(response)
    }

    async fn save_cache_async(&self) -> Result<(), Box<dyn Error>> {
        let serialized = serde_json::to_string_pretty(&self.cache)?;

        tokio::fs::write(CACHE_PATH, serialized.as_bytes()).await?;

        Ok(())
    }
}
