use std::collections::HashMap;

use eyre::ContextCompat;

use crate::{config::PlayerConfig, hiscore_lookup::Metric};

// The metrics that were updated since the last lookup
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

impl ExpUpdate {
    pub fn triggers_post(&self, player: &PlayerConfig) -> bool {
        if player.metrics_blacklist.contains(&self.name) {
            return false;
        }
        if self.end_level > self.start_level {
            player.metrics_whitelist.contains(&self.name) || player.show_levelups
        } else {
            // Don't show exp gains without levelups, even when whitelisted
            player.show_exp_gain
        }
    }

    pub fn update_message_part(&self) -> String {
        if self.end_level > self.start_level {
            format!(
                "level {} {} ({} exp)",
                self.end_level,
                self.name,
                self.end_exp - self.start_exp
            )
        } else {
            format!("{} {} exp", self.end_exp - self.start_exp, self.name)
        }
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

pub struct KcUpdate {
    pub name: String,
    pub start_kc: u32,
    pub end_kc: u32,
}

impl KcUpdate {
    pub fn triggers_post(&self, player: &PlayerConfig) -> bool {
        if player.metrics_blacklist.contains(&self.name) {
            return false;
        }
        player.metrics_whitelist.contains(&self.name) || player.show_kc_increases
    }

    pub fn update_message_part(&self) -> String {
        let delta = self.end_kc - self.start_kc;
        if self.name == "Collections Logged" {
            if delta == 1 {
                "a new collection log slot".to_string()
            } else {
                format!("{} new collection log slots", delta)
            }
        } else {
            format!("{} **{}** kc", delta, self.name)
        }
    }
}

impl MetricUpdates {
    pub fn is_empty(&self) -> bool {
        self.exp_updates.is_empty() && self.kc_updates.is_empty()
    }

    pub fn triggers_post(&self, player: &PlayerConfig) -> bool {
        self.exp_updates.iter().any(|u| u.triggers_post(player))
            || self.kc_updates.iter().any(|u| u.triggers_post(player))
    }

    pub fn get_full_update_message(&self, player: &PlayerConfig) -> String {
        let mut high_prio_parts: Vec<String> = Vec::new();
        let mut low_prio_parts: Vec<String> = Vec::new();

        for exp_update in self.exp_updates.iter() {
            if player.metrics_blacklist.contains(&exp_update.name) {
                continue;
            }
            let message_part = exp_update.update_message_part();
            if exp_update.end_level > exp_update.start_level {
                high_prio_parts.push(message_part);
            } else {
                low_prio_parts.push(message_part);
            }
        }

        for kc_update in self.kc_updates.iter() {
            if player.metrics_blacklist.contains(&kc_update.name) {
                continue;
            }
            let message_part = kc_update.update_message_part();
            if kc_update.name == "Collections Logged" || kc_update.name.contains("Clue Scrolls") {
                high_prio_parts.push(message_part);
            } else {
                low_prio_parts.push(message_part);
            }
        }

        match (high_prio_parts.is_empty(), low_prio_parts.is_empty()) {
            (true, true) => unreachable!(),
            (true, false) => {
                format!(
                    "{} just got {}{}!",
                    player.player_alias(),
                    join_messages(&low_prio_parts),
                    player.player_explanation
                )
            }
            (false, true) => {
                format!(
                    "{} just got {}{}!",
                    player.player_alias(),
                    join_messages(&high_prio_parts),
                    player.player_explanation
                )
            }
            (false, false) => {
                format!(
                    "{} just got {}{}! He also got {}.",
                    player.player_alias(),
                    join_messages(&high_prio_parts),
                    player.player_explanation,
                    join_messages(&low_prio_parts),
                )
            }
        }
    }
}

fn join_messages(messages: &[String]) -> String {
    let mut output = String::new();
    for (i, message) in messages.iter().enumerate() {
        output.push_str(message);
        if i == messages.len() - 1 {
        } else if i == messages.len() - 2 {
            output.push_str(", and ");
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

pub fn get_update_message(
    metric: &Metric,
    prev_metric: &Metric,
    player: &PlayerConfig,
) -> eyre::Result<Option<String>> {
    println!(
        "Metric updated for {}: {} -> {}",
        player.player_alias(),
        metric,
        prev_metric
    );
    if player.metrics_blacklist.contains(&metric.name) {
        return Ok(None);
    }
    if let Some(exp) = metric.exp {
        // This is an exp metric
        let exp_gained = exp - prev_metric.exp.context("No previous exp")?;
        if (player.show_levelups || player.metrics_whitelist.contains(&metric.name))
            && metric.score > prev_metric.score
        {
            Ok(Some(format!(
                "{} just got level {} {}{}!",
                player.player_alias(),
                metric.score,
                metric.name,
                player.player_explanation
            )))
        // Ignore whitelist for exp gain without levelups
        } else if player.show_exp_gain && exp_gained > 0 {
            Ok(Some(format!(
                "{} got {} {} exp{}!",
                player.player_alias(),
                exp_gained,
                metric.name,
                player.player_explanation
            )))
        } else {
            Ok(None)
        }
    } else {
        // This is a kc metric
        let delta = metric
            .score
            .checked_sub(prev_metric.score)
            .with_context(|| {
                format!(
                    "{} kc decreased {} -> {}",
                    metric.name, prev_metric.score, metric.score
                )
            })?;
        if player.show_kc_increases || player.metrics_whitelist.contains(&metric.name) {
            if metric.name == "Collections Logged" {
                if delta == 1 {
                    Ok(Some(format!(
                        "{} got a new collection log slot{}! What could it be?",
                        player.player_alias(),
                        player.player_explanation
                    )))
                } else {
                    Ok(Some(format!(
                        "{} got {} new collection log slots{}! What could they be?",
                        player.player_alias(),
                        delta,
                        player.player_explanation
                    )))
                }
            } else {
                Ok(Some(format!(
                    "{} increased **{}** kc{}: {} -> {}",
                    player.player_alias(),
                    metric.name,
                    player.player_explanation,
                    prev_metric.score,
                    metric.score
                )))
            }
        } else {
            Ok(None)
        }
    }
}

#[test]
fn test_update_message() {
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
    let metric_updates = MetricUpdates {
        exp_updates: vec![
            ExpUpdate {
                name: "Strength".to_string(),
                start_exp: 0,
                end_exp: 41_144,
                start_level: 99,
                end_level: 99,
            },
            ExpUpdate {
                name: "Hitpoints".to_string(),
                start_exp: 0,
                end_exp: 13_693,
                start_level: 99,
                end_level: 99,
            },
            ExpUpdate {
                name: "Farming".to_string(),
                start_exp: 0,
                end_exp: 1093,
                start_level: 99,
                end_level: 99,
            },
        ],
        kc_updates: vec![
            KcUpdate {
                name: "Collections Logged".to_string(),
                start_kc: 388,
                end_kc: 389,
            },
            KcUpdate {
                name: "Clue Scrolls (beginner)".to_string(),
                start_kc: 100,
                end_kc: 102,
            },
        ],
    };
    assert_eq!(
        metric_updates.get_full_update_message(&player),
        "Limpwurt just got a new collection log slot, and 2 **Clue Scrolls (beginner)** kc! He also got 41144 Strength exp, 13693 Hitpoints exp, and 1093 Farming exp."
    );
}
