use std::sync::Arc;

use anyhow::Result;
use mongodb::{
    bson::{doc, Document},
    options::{FindOneOptions, UpdateOptions},
};
use serde::{Deserialize, Serialize};
use twilight_model::{
    guild::audit_log::AuditLogEventType,
    id::{
        marker::{ChannelMarker, GuildMarker},
        Id,
    },
};

use crate::ctx::Context;

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct UserProfile {
// }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuildConfig {
    #[serde(rename = "_id")]
    pub id: Id<GuildMarker>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub welcomer: Option<WelcomerConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anti_abuse: Option<AntiAbuseConfig>,
}

impl GuildConfig {
    pub async fn get_guild(
        ctx: &Arc<Context>,
        guild_id: Id<GuildMarker>,
        options: Option<FindOneOptions>,
    ) -> Result<Option<GuildConfig>> {
        let configs = ctx
            .get_mongodb()
            .database("custos")
            .collection::<GuildConfig>("guild_configs");
        let guild_cfg = configs
            .find_one(doc! { "_id": guild_id.to_string() }, options)
            .await?;
        let config = GuildConfig {
            id: guild_id,
            welcomer: None,
            anti_abuse: None,
        };

        if guild_cfg.is_none() {
            configs.insert_one(config.clone(), None).await?;
            return Ok(Some(config));
        }

        Ok(guild_cfg)
    }

    pub async fn update_data_by_id_upsert(
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

    pub async fn update_data_upsert(&self, ctx: &Arc<Context>, update: Document) -> Result<()> {
        ctx.get_mongodb()
            .database("custos")
            .collection::<GuildConfig>("guild_configs")
            .update_one(
                doc! { "_id": self.id.to_string() },
                update,
                UpdateOptions::builder().upsert(true).build(),
            )
            .await?;
        Ok(())
    }
}

pub mod anti_abuse_punishment_action {
    pub const BAN: i32 = 1;
    pub const KICK: i32 = 2;
    pub const TIMEOUT: i32 = 4;
    pub const DEMOTE: i32 = 8;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AntiAbuseActionBuilder {
    #[serde(with = "flags_serde")]
    pub flags: i32,
}

pub mod flags_serde {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(flags: &i32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(*flags)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<i32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let flags: i32 = Deserialize::deserialize(deserializer)?;
        Ok(flags)
    }
}

impl AntiAbuseActionBuilder {
    #[inline]
    #[allow(dead_code)]
    pub fn new() -> AntiAbuseActionBuilder {
        AntiAbuseActionBuilder { flags: 0 }
    }

    #[inline]
    #[allow(dead_code)]
    pub fn add_ban(mut self) -> Self {
        self.flags |= anti_abuse_punishment_action::BAN;
        self
    }

    #[inline]
    #[allow(dead_code)]
    pub fn add_kick(mut self) -> Self {
        self.flags |= anti_abuse_punishment_action::KICK;
        self
    }

    #[inline]
    #[allow(dead_code)]
    pub fn add_timeout(mut self) -> Self {
        self.flags |= anti_abuse_punishment_action::TIMEOUT;
        self
    }

    #[inline]
    #[allow(dead_code)]
    pub fn add_demote(mut self) -> Self {
        self.flags |= anti_abuse_punishment_action::DEMOTE;
        self
    }

    #[inline]
    #[allow(dead_code)]
    pub fn is_ban(&self) -> bool {
        self.flags & anti_abuse_punishment_action::BAN == 0
    }

    #[inline]
    #[allow(dead_code)]
    pub fn is_kick(&self) -> bool {
        self.flags & anti_abuse_punishment_action::KICK == 0
    }

    #[inline]
    #[allow(dead_code)]
    pub fn is_timeout(&self) -> bool {
        self.flags & anti_abuse_punishment_action::TIMEOUT == 0
    }

    #[inline]
    #[allow(dead_code)]
    pub fn is_demote(&self) -> bool {
        self.flags & anti_abuse_punishment_action::DEMOTE == 0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WelcomerConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<Id<ChannelMarker>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AntiAbuseConfig {
    pub watched_actions: Vec<AntiAbuseEventConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AntiAbuseEventConfig {
    pub action_type: AuditLogEventType,
    pub max_sanctions: i32,
    pub sanction_cooldown: i32,
    pub punishment: AntiAbuseActionBuilder,
}
