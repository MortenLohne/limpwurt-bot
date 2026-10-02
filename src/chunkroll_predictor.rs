use chrono::{DateTime, Utc};
use eyre::ContextCompat;

use crate::hiscore_lookup::Metric;

pub fn predict_chunkroll_date(metrics: &[Metric]) -> eyre::Result<PredictionResult> {
    let start_crafting_exp = 8_242_610;
    let current_crafting_exp = metrics
        .iter()
        .find(|m| m.name == "Crafting")
        .context("Crafting metric not found")?
        .exp
        .context("No crafting exp found")?;
    let delta_crafting_exp = current_crafting_exp - start_crafting_exp;
    let target_crafting_exp = 13_034_431u32;
    // Banked materials (uncut gems etc.) are expected to run out around this point,
    // after which exp is assumed to come in at 1/3 of the rate from before
    let slowdown_crafting_exp = 11_000_000u32;
    let slowdown_factor = 3.0;

    // Exp after the slowdown point counts `slowdown_factor` times as much towards the time spent
    let weighted_exp = |exp: u32| {
        (exp.min(slowdown_crafting_exp) - start_crafting_exp) as f32
            + slowdown_factor * exp.saturating_sub(slowdown_crafting_exp) as f32
    };

    let grind_start = DateTime::parse_from_rfc3339("2026-09-30T23:00:00+02:00")?;
    let elapsed = Utc::now() - grind_start.to_utc();
    let days_elapsed = elapsed.as_seconds_f32() / 86_400.0;

    let ratio_done = delta_crafting_exp as f32 / (target_crafting_exp - start_crafting_exp) as f32;
    let time_ratio_done = weighted_exp(current_crafting_exp) / weighted_exp(target_crafting_exp);

    let crafting_exp_left = target_crafting_exp.saturating_sub(current_crafting_exp);
    let percent_done = 100.0 * ratio_done;
    let average_chunkroll_date = grind_start
        .to_utc()
        .checked_add_days(chrono::Days::new(
            (days_elapsed / time_ratio_done).ceil() as u64
        ))
        .context("Failed chrono days calculation")?;

    Ok(PredictionResult {
        crafting_exp_left,
        percent_done,
        average_chunkroll_date,
    })
}

pub struct PredictionResult {
    pub crafting_exp_left: u32,
    pub percent_done: f32,
    pub average_chunkroll_date: DateTime<Utc>,
}

impl PredictionResult {
    pub fn update_message(&self) -> String {
        format!(
            "Limpwurt needs another {}k crafting exp for level 99, and is {:.1}% done with the grind. Chunkroll is estimated on **{}**.",
            self.crafting_exp_left / 1000,
            self.percent_done,
            self.average_chunkroll_date.format("%d %B %Y")
        )
    }
}
