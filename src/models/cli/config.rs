use crate::core::entity_traits::config_file::ConfigFile;
use crate::models::config::recording_timeout::RecordingTimeoutConfig;
use crate::models::config::Config;
use crate::models::data::musicbrainz::mbid::mbid_kind::MBIDKind;
use crate::models::data::musicbrainz::mbid::MBID;
use crate::utils::extensions::chrono_ext::DurationExt;
use chrono::Duration;
use clap::Parser;
use clap::Subcommand;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct ConfigCli {
    #[command(subcommand)]
    pub command: ConfigCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ConfigCommands {
    /// Associate an user token to an username. This makes `--token` arguments optional, and prevent always having to insert it
    SetToken {
        /// Name of the user to add the token
        username: String,

        /// User token
        token: String,
    },

    /// Prevent the recording to appear on radios for a while. If you're burn out of a track and need it gone, use this.
    Timeout {
        /// A string containing a MBID of a recording
        recording: String,

        /// A duration to timeout for
        duration: String,
    },
}

impl ConfigCommands {
    pub async fn run(&self) -> color_eyre::Result<()> {
        match self {
            Self::SetToken { username, token } => {
                let mut conf = Config::load()?;
                conf.set_token(username.clone(), token.clone());
                conf.save()?;
            }

            Self::Timeout {
                recording,
                duration,
            } => {
                RecordingTimeoutConfig::set_timeout(
                    MBID::from_string(recording, MBIDKind::Recording)?.unwrap_recording(),
                    Duration::from_human_string(duration)?,
                )?;
            }
        }

        Ok(())
    }
}
