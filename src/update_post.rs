use eyre::ContextCompat;

use crate::{config::PlayerConfig, hiscore_lookup::Metric};

pub fn get_update_message(
    metric: &Metric,
    prev_metric: &Metric,
    player: &PlayerConfig,
) -> eyre::Result<Option<String>> {
    println!("Metric updated: {} -> {}", metric, prev_metric);
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
        } else if (player.show_exp_gain || player.metrics_whitelist.contains(&metric.name))
            && exp_gained > 0
        {
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
        if player.show_kc_increases || player.metrics_whitelist.contains(&metric.name) {
            if metric.name == "Collections Logged" {
                Ok(Some(format!(
                    "{} got a new collection log slot{}! What could it be?",
                    player.player_alias(),
                    player.player_explanation
                )))
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
