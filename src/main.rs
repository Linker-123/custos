use anyhow::Result;
use futures_util::{future::join_all, StreamExt};
use std::{env, iter, sync::Arc, thread};
use tokio::{signal, sync::watch, task::JoinSet};
use twilight_gateway::{
    stream::{self, ShardEventStream},
    CloseFrame, Config, Intents, Shard,
};

use crate::ctx::Context;

mod commands;
mod ctx;
mod events;
mod plugins;
mod schemas;
mod tags;

#[tokio::main]
async fn main() -> Result<()> {
    parallel_shards_init().await?;
    Ok(())
}

async fn parallel_shards_init() -> Result<()> {
    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN")?;
    let config = Config::new(
        token.clone(),
        Intents::GUILDS
            | Intents::GUILD_MESSAGES
            | Intents::GUILD_MEMBERS
            | Intents::MESSAGE_CONTENT,
    );
    let context = Arc::new(Context::new(&token).await?);
    context.register_commands().await?;

    let tasks = thread::available_parallelism()?.get();
    let init = iter::repeat_with(Vec::new)
        .take(tasks)
        .collect::<Vec<Vec<_>>>();
    let shards =
        stream::create_recommended(context.get_http(), config, |_, builder| builder.build())
            .await?
            .enumerate()
            .fold(init, |mut fold, (idx, shard)| {
                fold[idx % tasks].push(shard);
                fold
            });

    let (tx, rx) = watch::channel(false);
    let mut set = JoinSet::new();

    for mut shards in shards {
        let mut rx = rx.clone();
        let ctx = Arc::clone(&context);
        set.spawn(async move {
            tokio::select! {
                _ = listen_to_shards(shards.iter_mut(), Arc::clone(&ctx)) => {},
                _ = rx.changed() => {
                    join_all(shards.iter_mut().map(|shard| async move {
                        shard.close(CloseFrame::NORMAL).await
                    })).await;
                }
            }
        });
    }

    signal::ctrl_c().await?;

    tracing::debug!("shutting down");

    tx.send(true)?;
    while set.join_next().await.is_some() {}

    Ok(())
}

async fn listen_to_shards(shards: impl Iterator<Item = &mut Shard>, context: Arc<Context>) {
    let mut stream = ShardEventStream::new(shards);
    loop {
        let (shard, event) = match stream.next().await {
            Some((shard, Ok(event))) => (shard, event),
            Some((_, Err(source))) => {
                tracing::warn!(?source, "error receiving event");

                if source.is_fatal() {
                    break;
                }

                continue;
            }
            None => break,
        };

        let shard_id = shard.id();
        let event_kind = event.kind();
        context.get_cache().update(&event);

        let result = events::process_event(shard, event, &context).await;
        if let Err(e) = result {
            let e = e.to_string();
            tracing::error!(?event_kind, ?shard_id, error = e);
        }
    }
}
