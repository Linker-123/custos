use anyhow::{Error, Result};
use bson::doc;
use mongodb::options::FindOneOptions;
use std::sync::Arc;
use tracing::{debug, instrument, trace};
use twilight_http::request::AuditLogReason;
use twilight_model::{
    gateway::payload::incoming::GuildAuditLogEntryCreate,
    guild::Permissions,
    id::{
        marker::{GuildMarker, RoleMarker, UserMarker},
        Id,
    },
};

use crate::{
    ctx::Context,
    schemas::{AntiAbuseEventConfig, GuildConfig},
};

use self::schemas::AuditLogEntry;

use super::moderator;

pub async fn on_audit_log_create(
    context: &Arc<Context>,
    log_entry: Box<GuildAuditLogEntryCreate>,
) -> Result<()> {
    debug!("Received audit log entry {log:#?}", log = log_entry);
    let guild_id = log_entry.guild_id.unwrap(); // we unwrap because it's definitely present in this event.

    let moderator_id = match log_entry.user_id {
        Some(g) => g,
        None => return Err(Error::msg("No user_id field present.")),
    };

    if moderator_id.get() == context.get_app().id.get() {
        return Ok(());
    }

    let guild_config = GuildConfig::get_guild(
        context,
        guild_id,
        Some(
            FindOneOptions::builder()
                .projection(doc! { "anti_abuse": 1 })
                .build(),
        ),
    )
    .await?
    .unwrap();

    // TODO: use let-else
    let anti_abuse = match guild_config.anti_abuse {
        Some(cfg) => cfg,
        None => return Ok(()),
    };

    let action_log = anti_abuse
        .watched_actions
        .iter()
        .find(|event| event.action_type == log_entry.action_type);

    debug!("action log: {action_log:#?}");

    let action_log = match action_log {
        Some(action_config) => action_config,
        None => return Ok(()),
    };

    let audit_log_entry =
        AuditLogEntry::from_audit_log_entry(&log_entry, action_log.sanction_cooldown)?;
    audit_log_entry.insert(context).await?;

    let log_entry_count = audit_log_entry
        .count_entries_for(context, action_log.action_type)
        .await?;

    if log_entry_count > action_log.max_sanctions.try_into()? {
        if action_log.punishment.is_ban() {
            moderator::ban(
                context,
                guild_id,
                audit_log_entry.moderator_id,
                0,
                format!(
                    "User exceeded {} sanctions per {} seconds for the action type {:?}",
                    action_log.max_sanctions, action_log.sanction_cooldown, action_log.action_type
                ),
            )
            .await?;
        } else if action_log.punishment.is_kick() {
            moderator::kick(
                context,
                guild_id,
                audit_log_entry.moderator_id,
                format!(
                    "User exceeded {} sanctions per {} seconds for the action type {:?}",
                    action_log.max_sanctions, action_log.sanction_cooldown, action_log.action_type
                ),
            )
            .await?;
        } else {
            if action_log.punishment.is_timeout() {
                unimplemented!()
            }

            if action_log.punishment.is_demote() {
                demote_abuser(context, guild_id, audit_log_entry.moderator_id, action_log).await?;
            }
        }
    }

    Ok(())
}

#[instrument]
pub async fn demote_abuser(
    context: &Arc<Context>,
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
    action_log: &AntiAbuseEventConfig,
) -> Result<()> {
    // TODO: use let-else
    let mut guild_member_roles = match context.get_cache().member(guild_id, user_id) {
        Some(g) => g.roles().to_vec(),
        None => {
            trace!("No guild member in cache, we'll try to fetch them!");

            let member = context
                .get_http()
                .guild_member(guild_id, user_id)
                .await?
                .model()
                .await?;
            member.roles
        }
    };

    let mut fetch_roles = Vec::with_capacity(guild_member_roles.len());

    for role_id in &guild_member_roles {
        let role = context.get_cache().role(*role_id);
        if let Some(r) = role {
            fetch_roles.push(r.to_owned());
        }
    }

    const MOD_PERMS: &[Permissions] = &[
        Permissions::ADMINISTRATOR,
        Permissions::KICK_MEMBERS,
        Permissions::BAN_MEMBERS,
        Permissions::MANAGE_CHANNELS,
        Permissions::MANAGE_GUILD,
        Permissions::VIEW_AUDIT_LOG,
        Permissions::MANAGE_MESSAGES,
        Permissions::MENTION_EVERYONE,
        Permissions::VIEW_GUILD_INSIGHTS,
        Permissions::MUTE_MEMBERS,
        Permissions::DEAFEN_MEMBERS,
        Permissions::MOVE_MEMBERS,
        Permissions::MANAGE_NICKNAMES,
        Permissions::MANAGE_ROLES,
        Permissions::MANAGE_WEBHOOKS,
        Permissions::MANAGE_EMOJIS_AND_STICKERS,
        Permissions::MANAGE_EVENTS,
        Permissions::MANAGE_THREADS,
        Permissions::MODERATE_MEMBERS,
    ];

    let roles_to_remove = fetch_roles
        .into_iter()
        .filter(|r| {
            for perm in MOD_PERMS {
                if r.permissions.contains(*perm) {
                    return true;
                }
            }
            false
        })
        .map(|r| r.id)
        .collect::<Vec<Id<RoleMarker>>>();

    guild_member_roles.retain(|r| !roles_to_remove.contains(r));

    context
        .get_http()
        .update_guild_member(guild_id, user_id)
        .roles(&guild_member_roles)
        .reason(&format!(
            "User exceeded {} sanctions per {} seconds for the action type {:?}",
            action_log.max_sanctions, action_log.sanction_cooldown, action_log.action_type
        ))?
        .await?;

    Ok(())
}

pub mod schemas {
    use std::sync::Arc;

    use anyhow::{Error, Result};
    use bson::to_bson;
    use chrono::{DateTime, Duration, Utc};
    use mongodb::{bson::doc, results::InsertOneResult};
    use serde::{Deserialize, Serialize};
    use twilight_model::{
        gateway::payload::incoming::GuildAuditLogEntryCreate,
        guild::audit_log::AuditLogEventType,
        id::{
            marker::{GenericMarker, GuildMarker, UserMarker},
            Id,
        },
    };

    use crate::ctx::Context;

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct AuditLogEntry {
        pub guild_id: Id<GuildMarker>,
        pub moderator_id: Id<UserMarker>,
        pub action: ActionEntry,
        #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
        pub expires_at: DateTime<Utc>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct ActionEntry {
        pub kind: AuditLogEventType,
        pub reason: Option<String>,
        pub target_id: Option<Id<GenericMarker>>,
    }

    impl AuditLogEntry {
        pub async fn insert(&self, context: &Arc<Context>) -> Result<InsertOneResult> {
            let audit_log_entries = context
                .get_mongodb()
                .database("custos")
                .collection::<AuditLogEntry>("audit_log_entries");

            Ok(audit_log_entries.insert_one(self, None).await?)
        }

        pub async fn count_entries_for(
            &self,
            context: &Arc<Context>,
            action: AuditLogEventType,
        ) -> Result<u64> {
            let audit_log_entries = context
                .get_mongodb()
                .database("custos")
                .collection::<AuditLogEntry>("audit_log_entries");

            let count = audit_log_entries
                .count_documents(
                    doc! {
                        "guild_id": to_bson(&self.guild_id)?,
                        "moderator_id": to_bson(&self.moderator_id)?,
                        "action.kind": to_bson(&action)?
                    },
                    None,
                )
                .await?;

            Ok(count)
        }

        pub fn from_audit_log_entry(
            value: &GuildAuditLogEntryCreate,
            saction_cooldown: i32,
        ) -> Result<Self> {
            let guild_id = match value.guild_id {
                Some(g) => g,
                None => return Err(Error::msg("No guild_id field present.")),
            };

            let moderator_id = match value.user_id {
                Some(g) => g,
                None => return Err(Error::msg("No user_id field present.")),
            };

            let mut expires_at = Utc::now();
            expires_at += Duration::seconds(saction_cooldown.into());

            Ok(AuditLogEntry {
                guild_id,
                moderator_id,
                action: ActionEntry {
                    kind: value.action_type,
                    reason: value.reason.clone(),
                    target_id: value.target_id,
                },
                expires_at: Utc::now(),
            })
        }
    }
}
