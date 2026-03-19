use std::collections::HashMap;

use serde::Deserialize;
use serenity::all::ChannelId;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub players_to_track: Vec<String>,
    pub channels: HashMap<ChannelId, Vec<PlayerConfig>>,
}

impl Config {
    pub fn validate(&self) -> eyre::Result<()> {
        // Check that players mentioned in a channel config are also tracked
        for (channel_id, channel) in self.channels.iter() {
            for player_config in channel {
                if !self.players_to_track.contains(&player_config.player_name) {
                    eyre::bail!(
                        "Player {} is configured for channel id {}, but is not tracked on hiscores",
                        player_config.player_name,
                        channel_id
                    )
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerConfig {
    pub player_name: String,
    pub player_alias: Option<String>,
    pub player_explanation: String,
    pub show_exp_gain: bool,
    pub show_levelups: bool,
    pub show_kc_increases: bool,
    pub metrics_whitelist: Vec<String>,
    pub metrics_blacklist: Vec<String>,
}

impl PlayerConfig {
    pub fn player_alias(&self) -> &str {
        self.player_alias.as_ref().unwrap_or(&self.player_name)
    }
}
