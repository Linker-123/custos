use std::sync::Arc;

use anyhow::Result;
use twilight_http::request::AuditLogReason;
use twilight_model::id::{
    marker::{GuildMarker, UserMarker},
    Id,
};

use crate::ctx::Context;

pub async fn ban(
    context: &Arc<Context>,
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
    delete_message_seconds: u32,
    reason: String,
) -> Result<()> {
    let http = context.get_http();
    http.create_ban(guild_id, user_id)
        .delete_message_seconds(delete_message_seconds)?
        .reason(&reason)?
        .await?;

    Ok(())
}

pub async fn kick(
    context: &Arc<Context>,
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
    reason: String,
) -> Result<()> {
    let http = context.get_http();
    http.remove_guild_member(guild_id, user_id)
        .reason(&reason)?
        .await?;

    Ok(())
}

