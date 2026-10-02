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

    let grind_start = DateTime::parse_from_rfc3339("2026-09-30T23:00:00+02:00")?;
    let elapsed = Utc::now() - grind_start.to_utc();
    let days_elapsed = elapsed.as_seconds_f32() / 86_400.0;

    let ratio_done = delta_crafting_exp as f32 / (target_crafting_exp - start_crafting_exp) as f32;

    let crafting_exp_left = target_crafting_exp.saturating_sub(current_crafting_exp);
    let percent_done = 100.0 * ratio_done;
    let average_chunkroll_date = grind_start
        .to_utc()
        .checked_add_days(chrono::Days::new((days_elapsed / ratio_done).ceil() as u64))
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
            "Limpwurt need another {}k crafting exp, and is {:.1}% done with the grind. Chunkroll is estimated on **{}**.",
            self.crafting_exp_left / 1000,
            self.percent_done,
            self.average_chunkroll_date.format("%d %B %Y")
        )
    }
}
