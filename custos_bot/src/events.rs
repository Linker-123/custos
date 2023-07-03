use anyhow::Result;
use custos_script::tokenizer::Tokenizer;
use custos_script::{
    bytecode::{BuiltInMethod, Constant, Function, FunctionType, Instruction},
    compiler::Compiler,
    parser::Parser,
    vm::VirtualMachine,
};
use std::rc::Rc;
use std::sync::Arc;
use twilight_gateway::{stream::ShardRef, Event};

use twilight_model::{
    application::interaction::{InteractionData, InteractionType},
    gateway::payload::{
        incoming::{GuildCreate, MemberChunk},
        outgoing::RequestGuildMembers,
    },
    id::Id,
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
            // tracing::info!("Message content: {content}", content = message.content);

            if message.content.starts_with("!eval ")
                && (message.author.id == Id::new(1072158687407378496)
                    || message.author.id == Id::new(778518819055861761))
            {
                let mut content = message.content.strip_prefix("!eval ").unwrap();
                content = content.trim();

                println!("Content = {}", content);

                let (args, mut content) = content.split_once("```").unwrap();
                let args = args
                    .split_whitespace()
                    .map(String::from)
                    .collect::<Vec<String>>();

                content = content.strip_prefix("```").unwrap_or(content);
                content = content.strip_suffix("```").unwrap_or(content);

                let content = content.to_string();
                let cid = message.channel_id;
                let http_client = context.http_sync.clone();
                rayon::spawn(move || {
                    let args = Rc::new(
                        args.into_iter()
                            .map(Constant::String)
                            .collect::<Vec<Constant>>(),
                    );

                    let http_client = Rc::new(http_client);
                    let tokenizer = Tokenizer::new(&content);
                    let mut parser = match Parser::new(tokenizer, &content) {
                        Ok(p) => p,
                        Err(e) => {
                            http_client.create_message(cid, &format!("```{}```", e));
                            return;
                        }
                    };
                    match parser.parse() {
                        Ok(_) => (),
                        Err(e) => {
                            http_client.create_message(cid, &format!("```{}```", e));
                            return;
                        }
                    };

                    let compiler = Compiler::default();
                    let mut chunk = compiler.compile_non_boxed(parser.declarations);

                    chunk.add_instruction(Instruction::GetGlobal("main".to_string()), 1);
                    chunk.add_instruction(Instruction::Call(0), 1);
                    chunk.add_instruction(Instruction::Return, 1);

                    let mut vm = VirtualMachine::new(Function {
                        arity: 0,
                        chunk,
                        name: "".to_owned(),
                        kind: FunctionType::Script,
                    });

                    let http_clone = Rc::clone(&http_client);
                    vm.define_built_in_fn(BuiltInMethod::new(
                        "send".to_owned(),
                        Rc::new(move |args| {
                            if let Some(Constant::String(message_content)) = args.get(0) {
                                let result = http_clone.create_message(cid, message_content);
                                return Constant::String(result.id);
                            }

                            Constant::None
                        }),
                        0u8,
                    ));

                    // let clone = Rc::clone(&args);
                    let clone_1 = Rc::clone(&args);
                    vm.define_built_in_fn(BuiltInMethod::new(
                        "get_args".to_owned(),
                        Rc::new(move |_| {
                            let data = Rc::clone(&clone_1);
                            Constant::Array(data)
                        }),
                        0,
                    ));

                    if let Some(err) = vm.interpret() {
                        http_client.create_message(cid, &format!("```{}```", err));
                    }
                });
            }
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
                        PingCommand::on_command_call(shard, context, inter, command_data).await?;
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
                InteractionData::MessageComponent(msg_comp) => {
                    if msg_comp
                        .custom_id
                        .starts_with(AntiAbuseCommand::get_component_tag())
                    {
                        AntiAbuseCommand::on_component_event(shard, context, inter, msg_comp)
                            .await?;
                    }
                }
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
