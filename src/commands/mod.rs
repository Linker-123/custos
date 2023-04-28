use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use twilight_gateway::stream::ShardRef;
use twilight_model::{
    application::{
        command::Command,
        interaction::{
            application_command::CommandData, message_component::MessageComponentInteractionData,
            modal::ModalInteractionData,
        },
    },
    gateway::payload::incoming::InteractionCreate,
};

use crate::ctx::Context;

pub mod anti_abuse;
pub mod debug;
pub mod welcomer;

#[async_trait]
pub trait CustosCommand {
    fn get_command_name() -> String;

    fn get_command_info() -> Command;

    async fn on_command_call(
        _shard: ShardRef<'_>,
        _context: &Arc<Context>,
        _inter: Box<InteractionCreate>,
        _command_data: Box<CommandData>,
    ) -> Result<()> {
        Ok(())
    }

    async fn on_autocomplete_call(
        _shard: ShardRef<'_>,
        _context: &Arc<Context>,
        _inter: Box<InteractionCreate>,
        _command_data: Box<CommandData>,
    ) -> Result<()> {
        Ok(())
    }

    async fn on_context_menu_call(
        _message_component: MessageComponentInteractionData,
    ) -> Result<()> {
        Ok(())
    }

    async fn on_modal_submit(_modal_data: ModalInteractionData) -> Result<()> {
        Ok(())
    }
}
