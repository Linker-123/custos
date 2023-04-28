use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use bson::{doc, to_bson};
use lazy_static::lazy_static;
use tracing::error_span;
use twilight_gateway::stream::ShardRef;
use twilight_model::{
    application::{
        command::{CommandOptionChoice, CommandOptionChoiceValue, CommandType},
        interaction::application_command::{CommandData, CommandDataOption, CommandOptionValue},
    },
    gateway::payload::incoming::InteractionCreate,
    guild::audit_log::AuditLogEventType,
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::{
    command::{
        CommandBuilder, IntegerBuilder, StringBuilder, SubCommandBuilder, SubCommandGroupBuilder,
    },
    InteractionResponseDataBuilder,
};

use super::CustosCommand;
use crate::{
    ctx::Context,
    plugins::anti_abuse::schemas::AuditLogEntry,
    schemas::{AntiAbuseActionBuilder, AntiAbuseEventConfig, GuildConfig},
    util,
};

lazy_static! {
    pub static ref ACTION_LABELS: Vec<(String, u16)> = vec![
        ("Guild Update", 1),
        ("Channel Create", 10),
        ("Channel Update", 11),
        ("Channel Delete", 12),
        ("Channel Overwrite Create", 13),
        ("Channel Overwrite Update", 14),
        ("Channel Overwrite Delete", 15),
        ("Member Kick", 20),
        ("Member Prune", 21),
        ("Member Ban Add", 22),
        ("Member Ban Remove", 23),
        ("Member Update", 24),
        ("Member Role Update", 25),
        ("Member Move", 26),
        ("Member Disconnect", 17),
        ("Bot Add", 28),
        ("Role Create", 30),
        ("Role Update", 31),
        ("Role Delete", 32),
        ("Invite Create", 40),
        ("Invite Update", 41),
        ("Invite Delete", 42),
        ("Webhook Create", 50),
        ("Webhook Update", 51),
        ("Webhook Delete", 52),
        ("Emoji Create", 60),
        ("Emoji Update", 61),
        ("Emoji Delete", 62),
        ("Message Delete", 72),
        ("Message BulkDelete", 73),
        ("Message Pin", 74),
        ("Message Unpin", 75),
        ("Integration Create", 80),
        ("Integration Update", 81),
        ("Integration Delete", 82),
        ("Stage Instance Create", 83),
        ("Stage Instance Update", 84),
        ("Stage Instance Delete", 85),
        ("Sticker Create", 90),
        ("Sticker Update", 91),
        ("Sticker Delete", 92),
        ("Guild Scheduled Event Create", 100),
        ("Guild Scheduled Event Update", 101),
        ("Guild Scheduled Event Delete", 102),
        ("Thread Create", 110),
        ("Thread Update", 111),
        ("Thread Delete", 112),
        ("Application Command Permission Update", 121),
        ("Auto Moderation Rule Create", 140),
        ("Auto Moderation Rule Update", 141),
        ("Auto Moderation Rule Delete", 142),
        ("Auto Moderation Block Message", 143),
        ("Auto Moderation Flag To Channel", 144),
        ("Auto Moderation User Communication Disabled", 145),
    ]
    .into_iter()
    .map(|(name, code)| {
        let mut name = name.to_lowercase();
        if name.contains("guild") {
            name = name.replace("guild", "server");
        }
        (name.to_lowercase(), code)
    })
    .collect::<Vec<(String, u16)>>();
}

pub struct AntiAbuseCommand {}

#[async_trait]
impl CustosCommand for AntiAbuseCommand {
    fn get_command_name() -> String {
        "anti-abuse".to_owned()
    }

    fn get_command_info() -> twilight_model::application::command::Command {
        CommandBuilder::new(
            Self::get_command_name(),
            "Configure anti-abuse plugin.",
            CommandType::ChatInput,
        )
        .option(
            SubCommandGroupBuilder::new("action", "Manage the watched actions.").subcommands([
                SubCommandBuilder::new("add", "Add a watched action.")
                    .option(
                        StringBuilder::new("action_type", "Set the action type to watch for")
                            .autocomplete(true)
                            .required(true),
                    )
                    .option(
                        IntegerBuilder::new(
                            "max_sanctions",
                            "Set the maximum amount of sanctions (amount of times the action can be performed)."
                        )
                            .min_value(0)
                            .max_value(128)
                            .required(true)
                    )
                    .option(
                        IntegerBuilder::new(
                            "sanction_cooldown",
                            "The time frame between which the sanctions will be recorded, after the timeframe the sanctions reset"
                        )
                        .min_value(60)
                        .max_value(3600)
                        .required(true)
                    ),
                SubCommandBuilder::new("remove", "Remove a watched action.")
                    .option(
                        StringBuilder::new("action_type", "Set the action type to watch for")
                            .autocomplete(true)
                            .required(true)
                    )
            ]),
        )
        .build()
    }

    async fn on_command_call(
        shard: ShardRef<'_>,
        context: &Arc<Context>,
        inter: Box<InteractionCreate>,
        data: Box<CommandData>,
    ) -> Result<()> {
        let guild_id = match inter.guild_id {
            Some(g) => g,
            None => return Ok(()),
        };

        let sub_command_group = &data.options[0];
        if sub_command_group.name != "action" {
            error_span!("Getting autcomplete for anti_abuse command that is not of sub command group type action.", shard = ?shard.id());
            return Ok(());
        }

        let sub_command = match &sub_command_group.value {
            CommandOptionValue::SubCommandGroup(d) => &d[0],
            _ => unreachable!(),
        };

        if sub_command.name == "add" {
            let options = match &sub_command.value {
                CommandOptionValue::SubCommand(sub_cmd) => sub_cmd,
                _ => unreachable!(),
            };

            let action_type = match &options[0].value {
                CommandOptionValue::String(s) => s,
                _ => unreachable!(),
            }
            .parse::<u16>()?;
            let max_sanctions = match &options[1].value {
                CommandOptionValue::Integer(s) => s,
                _ => unreachable!(),
            };
            let sanction_cooldown = match &options[2].value {
                CommandOptionValue::Integer(s) => s,
                _ => unreachable!(),
            };

            let guild_config = GuildConfig::get_guild(context, guild_id).await?.unwrap();

            guild_config
                .update_data_upsert(
                    context,
                    doc! {
                        "$push": {
                            "antiAbuse.watchedActions": to_bson(&AntiAbuseEventConfig {
                                action_type: AuditLogEventType::from(action_type),
                                max_sanctions: *max_sanctions as i32,
                                sanction_cooldown: *sanction_cooldown as i32,
                                punishment: AntiAbuseActionBuilder::new().add_ban()
                            })?
                        }
                    },
                )
                .await?;

            let interactions = context.get_interactions();
            util::send(
                &interactions,
                &inter,
                InteractionResponseType::ChannelMessageWithSource,
                InteractionResponseDataBuilder::new()
                    .content("Added new rule!")
                    .build(),
            )
            .await?;
        }

        Ok(())
    }

    async fn on_autocomplete_call(
        shard: ShardRef<'_>,
        context: &Arc<Context>,
        inter: Box<InteractionCreate>,
        data: Box<CommandData>,
    ) -> Result<()> {
        let sub_command_group = &data.options[0];
        if sub_command_group.name != "action" {
            error_span!("Getting autcomplete for anti_abuse command that is not of sub command group type action.", shard = ?shard.id());
            return Ok(());
        }

        let sub_command = match &sub_command_group.value {
            CommandOptionValue::SubCommandGroup(d) => &d[0],
            _ => unreachable!(),
        };

        let actual_value = match &sub_command.value {
            CommandOptionValue::SubCommand(options) => {
                let option = &options[0];
                match &option.value {
                    CommandOptionValue::Focused(value, kind) => (value, kind),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        };

        let query = actual_value.0.to_lowercase();
        let matching_labels = if query.is_empty() {
            ACTION_LABELS
                .iter()
                .take(25)
                .collect::<Vec<&(String, u16)>>()
        } else {
            ACTION_LABELS
                .iter()
                .filter(|(label, _)| label.starts_with(&query))
                .collect::<Vec<&(String, u16)>>()
        };

        let interactions = context.get_interactions();

        interactions
            .create_response(
                inter.id,
                &inter.token,
                &InteractionResponse {
                    kind: InteractionResponseType::ApplicationCommandAutocompleteResult,
                    data: Some(
                        InteractionResponseDataBuilder::new()
                            .choices(matching_labels.into_iter().map(|(label, code)| {
                                CommandOptionChoice {
                                    name: label.clone(),
                                    name_localizations: None,
                                    value: CommandOptionChoiceValue::String(code.to_string()),
                                }
                            }))
                            .build(),
                    ),
                },
            )
            .await?;

        Ok(())
    }
}
