use std::collections::HashMap;

use crate::{config::PlayerConfig, hiscore_lookup::Metric};

// The metrics that were updated since the last lookup
#[derive(Debug)]
pub struct MetricUpdates {
    pub exp_updates: Vec<ExpUpdate>,
    pub kc_updates: Vec<KcUpdate>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ExpUpdate {
    pub name: String,
    pub start_exp: u32,
    pub end_exp: u32,
    pub start_level: u32,
    pub end_level: u32,
}

/// The hiscores metric for collection log slots, which gets its own message
const COLLECTIONS_LOGGED: &str = "Collections Logged";

impl ExpUpdate {
    pub fn exp_gained(&self) -> u32 {
        self.end_exp.saturating_sub(self.start_exp)
    }

    /// Whether this update should be included in the levelup message
    pub fn shows_levelup(&self, player: &PlayerConfig) -> bool {
        self.end_level > self.start_level
            && !player.metrics_blacklist.contains(&self.name)
            && (player.show_levelups || player.metrics_whitelist.contains(&self.name))
    }

    /// Whether this update should be included in the exp gain message, regardless of levelups
    pub fn shows_exp_gain(&self, player: &PlayerConfig) -> bool {
        // The whitelist only enables levelups, not exp gains
        self.exp_gained() > 0
            && player.show_exp_gain
            && !player.metrics_blacklist.contains(&self.name)
    }
}

impl Ord for ExpUpdate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.end_level - self.start_level)
            .cmp(&(other.end_level - other.start_level))
            .reverse()
            .then(
                (self.end_exp - self.start_exp)
                    .cmp(&(other.end_exp - other.start_exp))
                    .reverse(),
            )
    }
}

impl PartialOrd for ExpUpdate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
pub struct KcUpdate {
    pub name: String,
    pub start_kc: u32,
    pub end_kc: u32,
}

impl KcUpdate {
    pub fn kc_gained(&self) -> u32 {
        self.end_kc.saturating_sub(self.start_kc)
    }

    /// Whether this update should be included in the kc or collection log message
    pub fn is_shown(&self, player: &PlayerConfig) -> bool {
        !player.metrics_blacklist.contains(&self.name)
            && (player.show_kc_increases || player.metrics_whitelist.contains(&self.name))
    }
}

impl MetricUpdates {
    pub fn is_empty(&self) -> bool {
        self.exp_updates.is_empty() && self.kc_updates.is_empty()
    }

    pub fn metric_was_updated(&self, name: &str) -> bool {
        self.exp_updates.iter().any(|u| u.name == name)
            || self.kc_updates.iter().any(|u| u.name == name)
    }

    /// The Discord messages to post for these updates, in posting order. There is a separate
    /// message for levelups, exp gains, kc increases and collection log slots, each including only
    /// the metrics that are enabled for the player. Empty if nothing should be posted.
    pub fn update_messages(&self, player: &PlayerConfig) -> Vec<String> {
        [
            self.levelup_message(player),
            self.exp_gain_message(player),
            self.kc_message(player),
            self.collection_log_message(player),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    fn levelup_message(&self, player: &PlayerConfig) -> Option<String> {
        let levelups: Vec<String> = self
            .exp_updates
            .iter()
            .filter(|u| u.shows_levelup(player))
            .map(|u| format!("level {} {}", u.end_level, u.name))
            .collect();
        if levelups.is_empty() {
            return None;
        }
        Some(format!(
            "{} just got {}{}!",
            player.player_alias(),
            join_messages(&levelups),
            player.player_explanation
        ))
    }

    fn exp_gain_message(&self, player: &PlayerConfig) -> Option<String> {
        let mut exp_updates: Vec<&ExpUpdate> = self
            .exp_updates
            .iter()
            .filter(|u| u.shows_exp_gain(player))
            .collect();
        if exp_updates.is_empty() {
            return None;
        }
        // Biggest exp gains first, regardless of levelups
        exp_updates.sort_by_key(|u| std::cmp::Reverse(u.exp_gained()));
        let exp_gains: Vec<String> = exp_updates
            .iter()
            .map(|u| format!("{} {} exp", u.exp_gained(), u.name))
            .collect();
        Some(format!(
            "{} got {}{}!",
            player.player_alias(),
            join_messages(&exp_gains),
            player.player_explanation
        ))
    }

    fn kc_message(&self, player: &PlayerConfig) -> Option<String> {
        let kc_updates: Vec<&KcUpdate> = self
            .kc_updates
            .iter()
            .filter(|u| u.name != COLLECTIONS_LOGGED && u.is_shown(player))
            .collect();
        match kc_updates.as_slice() {
            [] => None,
            [u] => Some(format!(
                "{} increased **{}** kc{}: {} -> {}",
                player.player_alias(),
                u.name,
                player.player_explanation,
                u.start_kc,
                u.end_kc
            )),
            _ => {
                let kcs: Vec<String> = kc_updates
                    .iter()
                    .map(|u| format!("**{}** {} -> {}", u.name, u.start_kc, u.end_kc))
                    .collect();
                Some(format!(
                    "{} increased kc{}: {}",
                    player.player_alias(),
                    player.player_explanation,
                    join_messages(&kcs)
                ))
            }
        }
    }

    fn collection_log_message(&self, player: &PlayerConfig) -> Option<String> {
        let update = self
            .kc_updates
            .iter()
            .find(|u| u.name == COLLECTIONS_LOGGED && u.is_shown(player))?;
        Some(match update.kc_gained() {
            1 => format!(
                "{} got a new collection log slot{}! What could it be?",
                player.player_alias(),
                player.player_explanation
            ),
            slots => format!(
                "{} got {} new collection log slots{}! What could they be?",
                player.player_alias(),
                slots,
                player.player_explanation
            ),
        })
    }
}

fn join_messages(messages: &[String]) -> String {
    let mut output = String::new();
    for (i, message) in messages.iter().enumerate() {
        output.push_str(message);
        if i == messages.len() - 1 {
        } else if i == messages.len() - 2 {
            // Only use the Oxford comma for 3 or more items
            output.push_str(if messages.len() == 2 {
                " and "
            } else {
                ", and "
            });
        } else {
            output.push_str(", ");
        }
    }
    output
}

pub fn metric_updates(metrics: &[Metric], prev_metrics: &HashMap<String, Metric>) -> MetricUpdates {
    let mut metric_update: MetricUpdates = MetricUpdates {
        exp_updates: Vec::new(),
        kc_updates: Vec::new(),
    };

    for metric in metrics {
        let Some(prev_metric) = prev_metrics.get(&metric.name) else {
            continue;
        };
        if !(metric.score > prev_metric.score
            || metric
                .exp
                .is_some_and(|exp| prev_metric.exp.is_some_and(|prev_exp| exp > prev_exp)))
        {
            continue;
        }
        if let Some((exp, prev_exp)) = metric.exp.zip(prev_metric.exp) {
            metric_update.exp_updates.push(ExpUpdate {
                name: metric.name.clone(),
                start_exp: prev_exp,
                end_exp: exp,
                start_level: prev_metric.score,
                end_level: metric.score,
            });
        } else {
            metric_update.kc_updates.push(KcUpdate {
                name: metric.name.clone(),
                start_kc: prev_metric.score,
                end_kc: metric.score,
            });
        }
    }

    metric_update.exp_updates.sort();

    metric_update
}

#[cfg(test)]
fn test_metric_updates() -> MetricUpdates {
    let exp_update = |name: &str, end_exp, start_level, end_level| ExpUpdate {
        name: name.to_string(),
        start_exp: 0,
        end_exp,
        start_level,
        end_level,
    };
    let kc_update = |name: &str, start_kc, end_kc| KcUpdate {
        name: name.to_string(),
        start_kc,
        end_kc,
    };
    MetricUpdates {
        exp_updates: vec![
            exp_update("Slayer", 36_390, 77, 78),
            exp_update("Strength", 41_144, 99, 99),
            exp_update("Hitpoints", 13_693, 99, 99),
            exp_update("Farming", 1093, 99, 99),
        ],
        kc_updates: vec![
            kc_update("Collections Logged", 388, 389),
            kc_update("Clue Scrolls (beginner)", 100, 102),
            kc_update("Obor", 10, 11),
        ],
    }
}

#[test]
fn test_update_messages() {
    let player = PlayerConfig {
        player_name: "OneChunkUp".to_string(),
        player_alias: Some("Limpwurt".to_string()),
        player_explanation: "".to_string(),
        show_exp_gain: true,
        show_levelups: true,
        show_kc_increases: true,
        metrics_whitelist: vec![],
        metrics_blacklist: vec![],
    };
    assert_eq!(
        test_metric_updates().update_messages(&player),
        [
            "Limpwurt just got level 78 Slayer!",
            "Limpwurt got 41144 Strength exp, 36390 Slayer exp, 13693 Hitpoints exp, and 1093 Farming exp!",
            "Limpwurt increased kc: **Clue Scrolls (beginner)** 100 -> 102 and **Obor** 10 -> 11",
            "Limpwurt got a new collection log slot! What could it be?",
        ]
    );
}

#[test]
fn test_update_messages_only_show_enabled_metrics() {
    let mut player = PlayerConfig {
        player_name: "OneChunkUp".to_string(),
        player_alias: Some("Limpwurt".to_string()),
        player_explanation: " on his chunk account".to_string(),
        show_exp_gain: true,
        show_levelups: false,
        show_kc_increases: false,
        metrics_whitelist: vec!["Slayer".to_string(), "Obor".to_string()],
        metrics_blacklist: vec!["Hitpoints".to_string()],
    };
    assert_eq!(
        test_metric_updates().update_messages(&player),
        [
            "Limpwurt just got level 78 Slayer on his chunk account!",
            "Limpwurt got 41144 Strength exp, 36390 Slayer exp, and 1093 Farming exp on his chunk account!",
            "Limpwurt increased **Obor** kc on his chunk account: 10 -> 11",
        ]
    );

    player.show_exp_gain = false;
    player.metrics_whitelist.clear();
    assert!(test_metric_updates().update_messages(&player).is_empty());
}
