use std::sync::Arc;

use anyhow::{Error, Result};
use async_trait::async_trait;
use mongodb::bson::doc;
use tracing::error_span;
use twilight_gateway::stream::ShardRef;
use twilight_http::client::InteractionClient;
use twilight_model::{
    application::{
        command::CommandType,
        interaction::application_command::{CommandData, CommandOptionValue},
    },
    channel::ChannelType,
    gateway::payload::incoming::InteractionCreate,
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::{
    command::{ChannelBuilder, CommandBuilder, StringBuilder, SubCommandBuilder},
    InteractionResponseDataBuilder,
};

use super::CustosCommand;
use crate::{ctx::Context, schemas::GuildConfig};

pub struct WelcomerCommand {}

#[async_trait]
impl CustosCommand for WelcomerCommand {
    fn get_command_name() -> String {
        "welcomer".to_owned()
    }

    fn get_command_info() -> twilight_model::application::command::Command {
        CommandBuilder::new(
            Self::get_command_name(),
            "Configure the welcomer plugin.",
            CommandType::ChatInput,
        )
        .option(
            SubCommandBuilder::new(
                "set-channel",
                "Set a channel the welcome message will be sent to.",
            )
            .option(
                ChannelBuilder::new("channel", "The welcome channel.")
                    .channel_types(vec![ChannelType::GuildText]),
            ),
        )
        .option(
            SubCommandBuilder::new(
                "set-message",
                "Set a welcome message to be sent. Using simple tags.",
            )
            .option(
                StringBuilder::new("value", "The welcome message.")
                    .min_length(1)
                    .max_length(2000),
            ),
        )
        .build()
    }

    async fn on_command_call(
        _: ShardRef<'_>,
        context: &Arc<Context>,
        inter: Box<InteractionCreate>,
        data: Box<CommandData>,
    ) -> Result<()> {
        let guild_id = match inter.guild_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let sub_command = &data.options[0];
        let options = match &sub_command.value {
            CommandOptionValue::SubCommand(scommand) => scommand,
            _ => return Ok(()),
        };

        let interactions = context.get_interactions();
        if sub_command.name == "set-channel" {
            // TODO: use let-else blocks when rustfmt supports it.
            let channel_id = match options.iter().find(|opt| opt.name == "channel") {
                Some(c) => match c.value {
                    CommandOptionValue::Channel(ch) => ch,
                    _ => return Err(Error::msg(
                        "Option with name 'channel' is not of CommandOptionValue::Channel type.",
                    )),
                },
                None => return Err(Error::msg("No 'channel' option found.")),
            };

            GuildConfig::set_welcomer_data(
                context,
                doc! { "$set": { "welcomer.channel_id": channel_id.to_string() } },
                guild_id,
            )
            .await?;

            interactions
                .create_response(
                    inter.id,
                    &inter.token,
                    &InteractionResponse {
                        kind: InteractionResponseType::ChannelMessageWithSource,
                        data: Some(
                            InteractionResponseDataBuilder::new()
                                .content(format!("Welcome channel set to <#{}>", channel_id))
                                .build(),
                        ),
                    },
                )
                .await?;
        } else if sub_command.name == "set-message" {
            let guild_config = match GuildConfig::get_guild(context, guild_id).await? {
                Some(g) => g,
                None => {
                    error_span!(
                        "on_command_call",
                        message = "get_guild did not return guild config."
                    );
                    return Err(Error::msg("get_guild did not return guild config."));
                }
            };

            async fn set_channel(
                interactions: InteractionClient<'_>,
                inter: Box<InteractionCreate>,
            ) -> Result<()> {
                interactions
                    .create_response(
                        inter.id,
                        &inter.token,
                        &InteractionResponse {
                            kind: InteractionResponseType::ChannelMessageWithSource,
                            data: Some(
                                InteractionResponseDataBuilder::new()
                                    .content("You have to set a welcome channel first.")
                                    .build(),
                            ),
                        },
                    )
                    .await?;
                Ok(())
            }

            if guild_config.welcomer.is_none() {
                return set_channel(interactions, inter).await;
            }

            let welcomer = match guild_config.welcomer {
                Some(w) => w,
                None => unreachable!(),
            };

            if welcomer.channel_id.is_none() {
                return set_channel(interactions, inter).await;
            }

            // TODO: use let-else blocks when rustfmt supports it.
            let message = match options.iter().find(|opt| opt.name == "value") {
                Some(c) => match &c.value {
                    CommandOptionValue::String(ch) => ch,
                    _ => return Err(Error::msg(
                        "Option with name 'channel' is not of CommandOptionValue::Channel type.",
                    )),
                },
                None => return Err(Error::msg("No 'channel' option found.")),
            };

            GuildConfig::set_welcomer_data(
                context,
                doc! { "$set": { "welcomer.message": message } },
                inter.guild_id.unwrap(),
            )
            .await?;

            interactions
                .create_response(
                    inter.id,
                    &inter.token,
                    &InteractionResponse {
                        kind: InteractionResponseType::ChannelMessageWithSource,
                        data: Some(
                            InteractionResponseDataBuilder::new()
                                .content("Welcome message has been set.")
                                .build(),
                        ),
                    },
                )
                .await?;
        }

        Ok(())
    }
}
