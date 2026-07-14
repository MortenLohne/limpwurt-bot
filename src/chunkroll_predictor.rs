use chrono::{DateTime, Utc};
use eyre::ContextCompat;

use crate::hiscore_lookup::Metric;

pub fn predict_chunkroll_date(metrics: &[Metric]) -> eyre::Result<PredictionResult> {
    let start_hp_exp = 60_114_667;
    let current_hp_exp = metrics
        .iter()
        .find(|m| m.name == "Hitpoints")
        .context("Hitpoints metric not found")?
        .exp
        .context("No hitpoints exp found")?;
    let delta_hp_exp = current_hp_exp - start_hp_exp;

    let start_clogs = 315;
    let current_clogs = metrics
        .iter()
        .find(|m| m.name == "Collections Logged")
        .context("Collections Logged metric not found")?
        .score;
    let delta_clogs = current_clogs.saturating_sub(start_clogs);

    let grind_start = DateTime::parse_from_rfc3339("2026-05-10T21:00:00+02:00")?;
    let elapsed = Utc::now() - grind_start.to_utc();
    let days_elapsed = elapsed.as_seconds_f32() / 86_400.0;

    let chaos_dwarf_hp_exp = 81;
    let chaos_dwarf_kc = delta_hp_exp / chaos_dwarf_hp_exp;

    // Each chaos dwarf has an effective 3/2560 chance to drop a Larran's key, so the
    // number of keys is Binomial(n = chaos_dwarf_kc, p = 3/2560). Use the normal
    // approximation for the 95% confidence interval (z = 1.96).
    let larrans_key_chance = 3.0 / 2560.0;
    let n = chaos_dwarf_kc as f32;
    let expected_larrans_keys = n * larrans_key_chance;
    let larrans_keys_sigma = (n * larrans_key_chance * (1.0 - larrans_key_chance)).sqrt();
    let larrans_keys_margin = 1.96 * larrans_keys_sigma;

    let muddy_keys = chaos_dwarf_kc as f32 / 18.29;
    let muddy_keys_per_day = muddy_keys / days_elapsed;

    let (_median_muddy_keys, avg_muddy_keys, lower_muddy_keys, upper_muddy_keys) = match delta_clogs
    {
        0 => (18915, 21934, 4114, 57403),
        1 => (14722, 17940, 2086, 52212),
        2 => (8326, 11979, 302, 43905),
        _ => (0, 0, 0, 0),
    };

    let average_chunkroll_date = Utc::now()
        .checked_add_days(chrono::Days::new(
            (avg_muddy_keys as f32 / muddy_keys_per_day).ceil() as u64,
        ))
        .context("Failed chrono days calculation")?;

    let lower_bound_chunkroll_date = Utc::now()
        .checked_add_days(chrono::Days::new(
            (lower_muddy_keys as f32 / muddy_keys_per_day).ceil() as u64,
        ))
        .context("Failed chrono days calculation")?;

    let upper_bound_chunkroll_date = Utc::now()
        .checked_add_days(chrono::Days::new(
            (upper_muddy_keys as f32 / muddy_keys_per_day).ceil() as u64,
        ))
        .context("Failed chrono days calculation")?;

    Ok(PredictionResult {
        clogs_left: 3u32.saturating_sub(delta_clogs),
        chaos_dwarf_kc,
        expected_larrans_keys,
        larrans_keys_margin,
        average_chunkroll_date,
        lower_bound_chunkroll_date,
        upper_bound_chunkroll_date,
    })
}

pub struct PredictionResult {
    pub clogs_left: u32,
    pub chaos_dwarf_kc: u32,
    pub expected_larrans_keys: f32,
    pub larrans_keys_margin: f32,
    pub average_chunkroll_date: DateTime<Utc>,
    pub lower_bound_chunkroll_date: DateTime<Utc>,
    pub upper_bound_chunkroll_date: DateTime<Utc>,
}
