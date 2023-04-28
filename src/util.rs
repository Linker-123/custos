use anyhow::Result;
use twilight_http::client::InteractionClient;
use twilight_model::{
    gateway::payload::incoming::InteractionCreate,
    http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
};

pub async fn send(
    interactions: &InteractionClient<'_>,
    inter: &InteractionCreate,
    kind: InteractionResponseType,
    data: InteractionResponseData,
) -> Result<()> {
    interactions
        .create_response(
            inter.id,
            &inter.token,
            &InteractionResponse {
                kind,
                data: Some(data),
            },
        )
        .await?;
    Ok(())
}
