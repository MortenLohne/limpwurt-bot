use std::collections::HashMap;

use serde::Deserialize;
use serenity::all::ChannelId;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub players_to_track: Vec<PlayerToTrack>,
    pub channels: HashMap<ChannelId, Vec<PlayerConfig>>,
}

impl Config {
    pub fn validate(&self) -> eyre::Result<()> {
        // Check that players mentioned in a channel config are also tracked
        for (channel_id, channel) in self.channels.iter() {
            for player_config in channel {
                if !self
                    .players_to_track
                    .iter()
                    .any(|p| p.name == player_config.player_name)
                {
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
pub struct PlayerToTrack {
    pub name: String,
    #[serde(rename = "type")]
    pub account_type: AccountType,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AccountType {
    Main,
    Ironman,
    UltimateIronman,
}

impl AccountType {
    pub fn highscore_url(&self) -> &str {
        match self {
            AccountType::Main => "hiscore_oldschool",
            AccountType::Ironman => "hiscore_oldschool_ironman",
            AccountType::UltimateIronman => "hiscore_oldschool_ultimate",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerConfig {
    pub player_name: String,
    pub player_alias: Option<String>,
    #[serde(default)]
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

#[test]
fn validate_test_config() {
    let config_str = r#"{
      "playersToTrack": [{ "name": "OneChunkUp", "type": "main" }],
      "channels": {
        "812770607527231488": [
          {
            "playerName": "OneChunkUp",
            "playerAlias": "Limpwurt",
            "showExpGain": false,
            "showLevelups": true,
            "showKcIncreases": true,
            "metricsWhitelist": [],
            "metricsBlacklist": [
              "Overall",
              "Clue Scrolls (all)",
              "Brutus",
              "The Royal Titans"
            ]
          }
        ]
      }
    }"#;
    let config: Config = serde_json::from_str(config_str).unwrap();
    assert!(config.validate().is_ok());
}
