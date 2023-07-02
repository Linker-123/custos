use std::{time::Duration};

use anyhow::Result;
use config::Config;
use mongodb::{
    bson::doc,
    options::{ClientOptions, IndexOptions},
    Client as MongoClient, IndexModel,
};

use twilight_cache_inmemory::InMemoryCache;
use twilight_http::{client::InteractionClient, Client as HttpClient};
use twilight_model::oauth::Application;

use crate::{
    commands::{
        anti_abuse::AntiAbuseCommand, debug::PingCommand, welcomer::WelcomerCommand, CustosCommand,
    },
    plugins::anti_abuse::schemas::AuditLogEntry,
    sync_http::SyncHttpClient,
};

#[derive(Debug)]
pub struct Context {
    pub cache: InMemoryCache,
    pub http: HttpClient,
    pub app: Application,
    pub mongodb: MongoClient,
    pub config: Config,
    pub http_sync: SyncHttpClient,
}

impl Context {
    pub async fn new(config: Config) -> Result<Self> {
        let token = config.get_string("token")?;
        let http_sync = SyncHttpClient::new(&token);
        let http = HttpClient::new(token);

        let app = http.current_user_application().await?.model().await?;

        let options = ClientOptions::parse_async(config.get_string("mongodb_address")?).await?;
        let mongodb = MongoClient::with_options(options)?;
        let context = Context {
            cache: InMemoryCache::new(),
            http,
            app,
            mongodb,
            config,
            http_sync,
        };

        context.register_indexes().await?;
        Ok(context)
    }

    pub async fn register_indexes(&self) -> Result<()> {
        let audit_log_entries = self
            .get_mongodb()
            .database(&self.get_config().get_string("db_name")?)
            .collection::<AuditLogEntry>("audit_log_entries");

        audit_log_entries
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "expires_at": 1 })
                    .options(
                        IndexOptions::builder()
                            .expire_after(Duration::from_secs(0))
                            .build(),
                    )
                    .build(),
                None,
            )
            .await?;

        Ok(())
    }

    #[inline]
    pub fn get_config(&self) -> &Config {
        &self.config
    }

    #[inline]
    pub fn get_mongodb(&self) -> &MongoClient {
        &self.mongodb
    }

    #[inline]
    pub fn get_http(&self) -> &HttpClient {
        &self.http
    }

    #[inline]
    pub fn get_interactions(&self) -> InteractionClient {
        self.get_http().interaction(self.get_app().id)
    }

    #[inline]
    pub fn get_app(&self) -> &Application {
        &self.app
    }

    #[inline]
    pub fn get_cache(&self) -> &InMemoryCache {
        &self.cache
    }

    pub async fn register_commands(&self) -> Result<()> {
        if self.get_config().get_bool("register_global_commands")? {
            let interactions_client = self.http.interaction(self.get_app().id);
            interactions_client.set_global_commands(&[]).await?;
            interactions_client
                .set_global_commands(&[
                    PingCommand::get_command_info(),
                    WelcomerCommand::get_command_info(),
                    AntiAbuseCommand::get_command_info(),
                ])
                .await?;
        }

        Ok(())
    }
}
