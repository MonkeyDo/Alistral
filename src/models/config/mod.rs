use crate::core::entity_traits::config_file::ConfigFile;
use clap::CommandFactory;
use derive_getters::Getters;
use listen_config::ListenConfig;
use mapper::MapperConfig;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

use super::cli::Cli;

pub mod global_config;
pub mod listen_config;
pub mod mapper;
pub mod recording_timeout;

#[derive(Debug, Serialize, Deserialize, Getters, Default)]
pub struct Config {
    /// Saved usertokens
    tokens: HashMap<String, String>,

    pub(super) mapper: Option<MapperConfig>,

    #[serde(default)]
    pub listens: ListenConfig,

    pub default_user: Option<String>,
}

impl Config {
    pub fn set_token(&mut self, username: String, token: String) {
        self.tokens.insert(username.to_lowercase(), token);
    }

    pub fn check_token(username: &str, arg: &Option<String>) -> String {
        if let Some(arg) = arg {
            return arg.clone();
        }

        match Self::load_or_panic().tokens.get(&username.to_lowercase()) {
            Some(val) => val.clone(),
            None => {
                Cli::command()
                    .error(
                        clap::error::ErrorKind::MissingRequiredArgument,
                        format!("No token was provided, and no tokens have been associated to {username}. Try adding your token to the command, or set the user's token with `config set-token <USERNAME> <TOKEN>`"),
                    )
                    .exit()
            }
        }
    }

    pub fn get_default_user() -> Option<String> {
        Self::load_or_panic().default_user
    }

    fn load_or_panic() -> Self {
        Self::load().expect("Cannot load the configuration file. Aborting.")
    }

    pub fn check_username(s: &Option<String>) -> String {
        if let Some(s) = s {
            return s.clone();
        }

        match Self::get_default_user() {
            Some(val) => val,
            None => {
                Cli::command()
                    .error(
                        clap::error::ErrorKind::MissingRequiredArgument,
                        "No username was provided, and the default username isn't set. Try adding your username to the command, or set the default username with `config default-user <USERNAME>`",
                    )
                    .exit()
            }
        }
    }
}

impl ConfigFile for Config {
    fn file_name() -> &'static str {
        "config.json"
    }
}
