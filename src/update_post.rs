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
