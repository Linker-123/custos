use anyhow::Result;
use clap::{Arg, Command};
use config::Config;

pub fn read_config() -> Result<Config> {
    let mut config = Config::builder()
        .add_source(config::File::with_name("config.toml").required(false))
        .set_default("db_name", "custos")?
        .set_default("mongodb_address", "mongodb://127.0.0.1:27017/")?;

    let matches = Command::new("hayat_online")
        .version("0.1")
        .about("Configure the options for hayat online")
        .arg(Arg::new("db_name").long("db").short('d'))
        .arg(Arg::new("mongodb_address").long("mongodb_address"))
        .arg(Arg::new("token").long("token"))
        .get_matches();

    let db_name = matches.get_one::<String>("db_name");
    let mongodb_address = matches.get_one::<String>("mongodb_address");
    let token = matches.get_one::<String>("token");

    if let Some(db_name) = db_name {
        config = config.set_override("db_name", db_name.clone())?;
    }

    if let Some(mongodb_address) = mongodb_address {
        config = config.set_override("mongodb_address", mongodb_address.clone())?;
    }

    if let Some(token) = token {
        config = config.set_override("token", token.clone())?;
    }
    Ok(config.build()?)
}
