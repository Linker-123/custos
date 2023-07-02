use serde::{Deserialize, Serialize};
use twilight_model::id::{marker::ChannelMarker, Id};

#[derive(Debug, Clone)]
pub struct SyncHttpClient {
    token: String,
}

#[derive(Serialize, Deserialize)]
pub struct MessageCreateResp {
    pub id: String,
}

impl SyncHttpClient {
    pub fn new(token: &str) -> SyncHttpClient {
        SyncHttpClient {
            token: token.to_owned(),
        }
    }

    pub fn create_message(
        &self,
        channel_id: Id<ChannelMarker>,
        content: &str,
    ) -> MessageCreateResp {
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(format!(
                "https://discord.com/api/v10/channels/{channel_id}/messages"
            ))
            .body(
                serde_json::json!({
                    "content": content
                })
                .to_string(),
            )
            .header("Authorization", format!("Bot {}", self.token))
            .header("Content-Type", "application/json")
            .send()
            .unwrap();
        response.json().unwrap()
    }
}
