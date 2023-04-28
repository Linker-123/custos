use std::sync::Arc;

use anyhow::Result;
use mongodb::{
    bson::{doc, Document},
    options::UpdateOptions,
};
use serde::{Deserialize, Serialize};
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker},
    Id,
};

use crate::ctx::Context;

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct UserProfile {
// }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuildConfig {
    #[serde(rename = "_id")]
    pub id: Id<GuildMarker>,
    pub welcomer: Option<WelcomerConfig>,
}

impl GuildConfig {
    pub async fn get_guild(
        ctx: &Arc<Context>,
        guild_id: Id<GuildMarker>,
    ) -> Result<Option<GuildConfig>> {
        let configs = ctx
            .get_mongodb()
            .database("custos")
            .collection::<GuildConfig>("guild_configs");
        let guild_cfg = configs
            .find_one(doc! { "_id": guild_id.to_string() }, None)
            .await?;
        let config = GuildConfig {
            id: guild_id,
            welcomer: None,
        };

        if guild_cfg.is_none() {
            configs.insert_one(config.clone(), None).await?;
            return Ok(Some(config));
        }

        Ok(guild_cfg)
    }

    pub async fn set_welcomer_data(
        ctx: &Arc<Context>,
        update: Document,
        guild_id: Id<GuildMarker>,
    ) -> Result<()> {
        ctx.get_mongodb()
            .database("custos")
            .collection::<GuildConfig>("guild_configs")
            .update_one(
                doc! { "_id": guild_id.to_string() },
                update,
                UpdateOptions::builder().upsert(true).build(),
            )
            .await?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WelcomerConfig {
    pub channel_id: Option<Id<ChannelMarker>>,
    pub message: Option<String>,
}
