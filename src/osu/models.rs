use serde::{Deserialize, Serialize};

// https://osu.ppy.sh/docs/index.html#chatchannel
#[derive(Serialize, Deserialize, Debug)]
pub struct ChatChannel {
    pub channel_id: u64,
    pub name: String,
    pub description: Option<String>,
}

// https://osu.ppy.sh/docs/index.html#chatmessage
#[derive(Serialize, Deserialize, Debug)]
pub struct ChatMessage {}

#[derive(Serialize, Deserialize, Debug)]
pub struct RefreshAccessTokenRequestBody {
    pub client_id: u64,
    pub client_secret: String,
    pub grant_type: String,
    pub refresh_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RefreshAccessTokenResponseBody {
    pub access_token: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub token_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreatePmRequestBody {
    pub target_id: u64,
    pub message: String,
    pub is_action: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
}

// https://osu.ppy.sh/docs/index.html#create-new-pm
// Тут почему-то в примере массив каналов, но должен быть один же?
// апд, по факту приходит один.
#[derive(Serialize, Deserialize, Debug)]
pub struct CreatePmResponseBody {
    pub channel: ChatChannel,
    pub message: ChatMessage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SendMessageToChannelRequestBody {
    pub message: String,
    pub is_action: bool,
}

// https://osu.ppy.sh/docs/index.html#send-message-to-channel
pub type SendMessageToChannelResponseBody = ChatMessage;
