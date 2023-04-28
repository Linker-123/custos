use anyhow::Result;
use mongodb::{options::ClientOptions, Client as MongoClient};
use twilight_cache_inmemory::InMemoryCache;
use twilight_http::{client::InteractionClient, Client as HttpClient};
use twilight_model::{id::Id, oauth::Application};

use crate::commands::{
    anti_abuse::AntiAbuseCommand, debug::PingCommand, welcomer::WelcomerCommand, CustosCommand,
};

#[derive(Debug)]
pub struct Context {
    cache: InMemoryCache,
    http: HttpClient,
    app: Application,
    mongodb: MongoClient,
}

impl Context {
    pub async fn new(token: &str) -> Result<Self> {
        let http = HttpClient::new(token.to_owned());
        let app = http.current_user_application().await?.model().await?;

        let options =
            ClientOptions::parse_async("mongodb://localhost:27017/?replicaSet=serverRepl").await?;
        let mongodb = MongoClient::with_options(options)?;

        Ok(Context {
            cache: InMemoryCache::new(),
            http,
            app,
            mongodb,
        })
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
        let interactions_client = self.http.interaction(self.get_app().id);
        interactions_client.set_global_commands(&[]).await?;
        interactions_client
            .set_guild_commands(
                Id::new(1099391006069243976),
                &[
                    PingCommand::get_command_info(),
                    WelcomerCommand::get_command_info(),
                    AntiAbuseCommand::get_command_info(),
                ],
            )
            .await?;

        Ok(())
    }
}
