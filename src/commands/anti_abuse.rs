use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use bson::{doc, to_bson};
use lazy_static::lazy_static;
use mongodb::options::FindOneOptions;
use tracing::{error_span, warn_span};
use twilight_gateway::stream::ShardRef;
use twilight_model::{
    application::{
        command::{CommandOptionChoice, CommandOptionChoiceValue, CommandType},
        interaction::{
            application_command::{CommandData, CommandOptionValue},
            message_component::MessageComponentInteractionData,
        },
    },
    channel::message::{
        component::{ActionRow, SelectMenu, SelectMenuOption},
        Component,
    },
    gateway::payload::incoming::InteractionCreate,
    guild::{audit_log::AuditLogEventType, Permissions},
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::{
    builder::{
        command::{
            CommandBuilder, IntegerBuilder, StringBuilder, SubCommandBuilder,
            SubCommandGroupBuilder,
        },
        InteractionResponseDataBuilder,
    },
    permission_calculator::PermissionCalculator,
};

use super::CustosCommand;
use crate::{
    ctx::Context,
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
    pub static ref ACTION_MENU_OPTIONS: Vec<SelectMenuOption> = vec![
        SelectMenuOption {
            default: true,
            description: Some("Remove all powerful roles of the user.".to_owned(),),
            emoji: None,
            label: "Demote".to_owned(),
            value: String::from("action-demote"),
        },
        SelectMenuOption {
            default: false,
            description: Some("Timeout the user.".to_owned()),
            emoji: None,
            label: "Timeout".to_owned(),
            value: String::from("action-timeout"),
        },
        SelectMenuOption {
            default: false,
            description: Some("Kick the user.".to_owned()),
            emoji: None,
            label: "Kick".to_owned(),
            value: String::from("action-kick"),
        },
        SelectMenuOption {
            default: false,
            description: Some("Ban the user.".to_owned()),
            emoji: None,
            label: "Ban".to_owned(),
            value: String::from("action-ban"),
        },
    ];
}

pub struct AntiAbuseCommand {}

#[async_trait]
impl CustosCommand for AntiAbuseCommand {
    fn get_command_name() -> String {
        "anti-abuse".to_owned()
    }

    fn get_component_tag() -> &'static str {
        "ab"
    }

    fn get_command_info() -> twilight_model::application::command::Command {
        CommandBuilder::new(
            Self::get_command_name(),
            "Configure anti-abuse plugin.",
            CommandType::ChatInput,
        ).default_member_permissions(Permissions::MANAGE_GUILD)
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

    async fn on_component_event(
        _shard: ShardRef<'_>,
        context: &Arc<Context>,
        inter: Box<InteractionCreate>,
        component_data: MessageComponentInteractionData,
    ) -> Result<()> {
        let member = match &inter.member {
            Some(m) => m,
            None => return Ok(()),
        };

        let guild_id = inter.guild_id.unwrap();
        let user_id = match &member.user {
            Some(user) => user.id,
            None => return Ok(()),
        };

        // Guild-level @everyone role that, by default, allows everyone to view
        // channels.
        let everyone_role = Permissions::VIEW_CHANNEL;

        let mut member_roles = Vec::with_capacity(member.roles.len());
        for role_id in &member.roles {
            let role = context.get_cache().role(*role_id);
            if let Some(role) = role {
                member_roles.push((*role_id, role.permissions));
            }
        }

        let member_roles = member_roles.as_slice();

        let calculator = PermissionCalculator::new(guild_id, user_id, everyone_role, member_roles);
        let calculated_permissions = calculator.root();
        let interactions = context.get_interactions();

        let guild = match context.get_cache().guild(guild_id) {
            Some(g) => g,
            None => {
                warn_span!("Got an interaction but no guild in the cache.");
                return Ok(());
            }
        };

        let owner_id = guild.owner_id();
        drop(guild);

        if !calculated_permissions.contains(Permissions::MANAGE_GUILD) && user_id != owner_id {
            util::send(
                &interactions,
                &inter,
                InteractionResponseType::UpdateMessage,
                InteractionResponseDataBuilder::new()
                    .content(
                        "You do not have `Manage Server` permissions to configure this plugin.",
                    )
                    .components([Component::ActionRow(ActionRow {
                        components: vec![Component::SelectMenu(SelectMenu {
                            custom_id: component_data.custom_id,
                            disabled: true,
                            max_values: Some(2),
                            min_values: Some(1),
                            options: ACTION_MENU_OPTIONS.clone(),
                            placeholder: Some("Select a punishment".to_owned()),
                        })],
                    })])
                    .build(),
            )
            .await?;
            return Ok(());
        }

        if component_data.custom_id.starts_with("ab-a") {
            let data_values = component_data
                .custom_id
                .strip_prefix("ab-a")
                .unwrap()
                .split('-')
                .skip(1)
                .map(|v| v.parse::<i32>().unwrap_or(-1))
                .collect::<Vec<i32>>();
            let action_type = &data_values[0];
            let max_sanctions = &data_values[1];
            let sanction_cooldown = &data_values[2];

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

            let mut existing_index = None;
            let action_type = AuditLogEventType::from(*action_type as u16);

            if let Some(anti_abuse) = &guild_config.anti_abuse {
                existing_index = anti_abuse
                    .watched_actions
                    .iter()
                    .position(|action| action.action_type == action_type);
            }

            if let Some(index) = existing_index {
                guild_config
                    .update_data_upsert(
                        context,
                        doc! {
                            "$set": {
                                {format!("anti_abuse.watched_actions.{index}")}: to_bson(&AntiAbuseEventConfig {
                                    action_type,
                                    max_sanctions: *max_sanctions,
                                    sanction_cooldown: *sanction_cooldown,
                                    punishment: AntiAbuseActionBuilder::new().add_ban()
                                })?
                            }
                        },
                    )
                    .await?;
            } else {
                guild_config
                    .update_data_upsert(
                        context,
                        doc! {
                            "$push": {
                                "anti_abuse.watched_actions": to_bson(&AntiAbuseEventConfig {
                                    action_type,
                                    max_sanctions: *max_sanctions,
                                    sanction_cooldown: *sanction_cooldown,
                                    punishment: AntiAbuseActionBuilder::new().add_ban()
                                })?
                            }
                        },
                    )
                    .await?;
            }

            util::send(
                &interactions,
                &inter,
                InteractionResponseType::UpdateMessage,
                InteractionResponseDataBuilder::new()
                    .content("Added a new action to watch for!")
                    .components([Component::ActionRow(ActionRow {
                        components: vec![Component::SelectMenu(SelectMenu {
                            custom_id: component_data.custom_id,
                            disabled: true,
                            max_values: Some(2),
                            min_values: Some(1),
                            options: ACTION_MENU_OPTIONS.clone(),
                            placeholder: Some("Select a punishment".to_owned()),
                        })],
                    })])
                    .build(),
            )
            .await?;
        }
        Ok(())
    }

    async fn on_command_call(
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

            let interactions = context.get_interactions();
            util::send(
                &interactions,
                &inter,
                InteractionResponseType::ChannelMessageWithSource,
                InteractionResponseDataBuilder::new()
                    .content("Added new rule!")
                    .components([Component::ActionRow(ActionRow {
                        components: vec![Component::SelectMenu(SelectMenu {
                            custom_id: format!(
                                // Anti-abuse - add - action_type - max_sanctions - sanction_cooldown
                                "ab-a-{}-{}-{}",
                                action_type, max_sanctions, sanction_cooldown
                            ),
                            disabled: false,
                            max_values: Some(2),
                            min_values: Some(1),
                            options: ACTION_MENU_OPTIONS.clone(),
                            placeholder: Some("Select a punishment".to_owned()),
                        })],
                    })])
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
