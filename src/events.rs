use anyhow::Result;
use std::sync::Arc;
use twilight_gateway::{stream::ShardRef, Event};
use twilight_model::{
    application::interaction::{InteractionData, InteractionType},
    gateway::payload::{
        incoming::{GuildCreate, MemberChunk},
        outgoing::RequestGuildMembers,
    },
};

use crate::{
    commands::{
        anti_abuse::AntiAbuseCommand, debug::PingCommand, welcomer::WelcomerCommand, CustosCommand,
    },
    ctx::Context,
    plugins,
};

pub async fn process_event(
    shard: ShardRef<'_>,
    event: Event,
    context: &Arc<Context>,
) -> Result<()> {
    tracing::debug!(?event, shard = ?shard.id(), "Processing event");

    match &event {
        Event::GuildCreate(guild) => on_guild_create(shard, guild).await?,
        Event::MemberChunk(chunk) => on_member_chunk(shard, chunk, context).await?,
        Event::MessageCreate(message) => {
            tracing::info!("Message content: {content}", content = message.content);
        }
        Event::MemberAdd(member_add) => {
            plugins::welcomer::on_member_add(context, Box::clone(member_add).into()).await?;
        }
        Event::InteractionCreate(inter) => {
            context.get_cache().update(&event);

            let mut inter = Box::clone(inter);
            let data = inter.data.take().unwrap();

            match data {
                InteractionData::ApplicationCommand(command_data) => {
                    if command_data.name == PingCommand::get_command_name() {
                        // if inter.kind == InteractionType::ApplicationCommandAutocomplete {
                        //     PingCommand::on_autocomplete_call(shard, context, inter, command_data)
                        //         .await?;
                        // } else {
                        PingCommand::on_command_call(shard, context, inter, command_data).await?;
                        // }
                    } else if command_data.name == WelcomerCommand::get_command_name() {
                        WelcomerCommand::on_command_call(shard, context, inter, command_data)
                            .await?;
                    } else if command_data.name == AntiAbuseCommand::get_command_name() {
                        if inter.kind == InteractionType::ApplicationCommandAutocomplete {
                            AntiAbuseCommand::on_autocomplete_call(
                                shard,
                                context,
                                inter,
                                command_data,
                            )
                            .await?;
                        } else {
                            AntiAbuseCommand::on_command_call(shard, context, inter, command_data)
                                .await?;
                        }
                    }
                }
                InteractionData::MessageComponent(_msg_comp) => {}
                InteractionData::ModalSubmit(_modal) => {}
                _ => todo!(),
            }
        }
        Event::GuildAuditLogEntryCreate(log_entry) => {
            plugins::anti_abuse::on_audit_log_create(context, Box::clone(log_entry)).await?;
        }
        _ => (),
    }

    Ok(())
}

async fn on_member_chunk(
    shard: ShardRef<'_>,
    chunk: &MemberChunk,
    context: &Arc<Context>,
) -> Result<()> {
    context.get_cache().update(chunk);
    tracing::info!(
        "Shard {} received a member chunk of size: {}",
        shard.id(),
        chunk.members.len()
    );
    Ok(())
}

async fn on_guild_create(mut shard: ShardRef<'_>, guild: &GuildCreate) -> Result<()> {
    shard
        .command(&RequestGuildMembers::builder(guild.id).query("", Some(0)))
        .await?;
    Ok(())
}
