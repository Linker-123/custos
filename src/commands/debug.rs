use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use twilight_gateway::stream::ShardRef;
use twilight_model::{
    application::{command::CommandType, interaction::application_command::CommandData},
    gateway::payload::incoming::InteractionCreate,
    http::interaction::InteractionResponseType,
};
use twilight_util::builder::{command::CommandBuilder, InteractionResponseDataBuilder};

use super::CustosCommand;
use crate::{ctx::Context, util};

pub struct PingCommand {}

#[async_trait]
impl CustosCommand for PingCommand {
    fn get_command_name() -> String {
        "debug".to_owned()
    }

    fn get_command_info() -> twilight_model::application::command::Command {
        CommandBuilder::new(
            Self::get_command_name(),
            "Debugging information of Custos.",
            CommandType::ChatInput,
        )
        .build()
    }

    async fn on_command_call(
        shard: ShardRef<'_>,
        context: &Arc<Context>,
        inter: Box<InteractionCreate>,
        _: Box<CommandData>,
    ) -> Result<()> {
        let message = format!(
            "`Shard`: #{}\n`Avg latency`: {}\n`Application ID`: {}\n`Version`: {}",
            shard.id().number(),
            if let Some(dur) = shard.latency().average() {
                format!("{:.2?}", dur)
            } else {
                "Not available.".to_owned()
            },
            context.get_app().id,
            env!("CARGO_PKG_VERSION")
        );

        let interactions = context.get_interactions();
        util::send(
            &interactions,
            &inter,
            InteractionResponseType::ChannelMessageWithSource,
            InteractionResponseDataBuilder::new()
                .content(message)
                .build(),
        )
        .await?;
        Ok(())
    }
}
