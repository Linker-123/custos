use crate::tags;
use crate::{ctx::Context, schemas::GuildConfig};
use anyhow::{Error, Result};
use bson::doc;
use mongodb::options::FindOneOptions;
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::{error, instrument};
use twilight_model::gateway::payload::incoming::MemberAdd;
use twilight_model::{id::marker::GuildMarker, id::Id, user::User};

#[derive(Debug)]
pub struct WelcomerMemberAdd {
    guild_id: Id<GuildMarker>,
    user: User,
}

impl From<Box<MemberAdd>> for WelcomerMemberAdd {
    fn from(value: Box<MemberAdd>) -> Self {
        WelcomerMemberAdd {
            guild_id: value.guild_id,
            user: value.user.clone(),
        }
    }
}

#[instrument]
pub async fn on_member_add(context: &Arc<Context>, member_add: WelcomerMemberAdd) -> Result<()> {
    let guild_config = GuildConfig::get_guild(
        context,
        member_add.guild_id,
        Some(
            FindOneOptions::builder()
                .projection(doc! { "welcomer": 1 })
                .build(),
        ),
    )
    .await?
    .unwrap();

    if let Some(welcomer) = guild_config.welcomer {
        let guild = match context.get_cache().guild(member_add.guild_id) {
            Some(g) => g,
            None => {
                error!("Tried to get guild by guild_id from cache and failed");
                return Err(Error::msg("The guild is not in cache for some reason"));
            }
        };
        let guild_name = guild.name().to_owned();

        drop(guild);

        if welcomer.channel_id.is_some() && welcomer.message.is_some() {
            let values = BTreeMap::from([
                ("server_name".to_owned(), guild_name),
                ("user_id".to_owned(), member_add.user.id.to_string()),
                ("user_name".to_owned(), member_add.user.name),
                (
                    "user_discrim".to_owned(),
                    member_add.user.discriminator.to_string(),
                ),
            ]);

            context
                .get_http()
                .create_message(welcomer.channel_id.unwrap())
                .content(&tags::parse_simple_tags(welcomer.message.unwrap(), values))?
                .await?;
        }
    }

    Ok(())
}
