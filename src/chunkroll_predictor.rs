use chrono::{DateTime, TimeDelta, Utc};
use eyre::ContextCompat;

use crate::hiscore_lookup::Metric;

/// Start of the hellhound grind. His first hellhound kills were on the evening of 28 October,
/// but he only started scattering the vile ashes on the 29th.
const GRIND_START: &str = "2026-10-29T00:00:00+01:00";
/// Hiscores exp at `GRIND_START`
const START_HP_EXP: u32 = 75_573_483;
const START_PRAYER_EXP: u32 = 4_520_932;

const TARGET_HERBLORE_EXP: u32 = 13_034_431;
/// Assumed rate for the remaining herblore exp
const HERBLORE_EXP_PER_DAY: f64 = 100_000.0;

/// Average HP exp per kill of a 116 HP hellhound with Water Blast and the twinflame staff. The
/// second hit of each cast deals 40% of the first hit (rounded down) and gives no exp, and exp is
/// only given for damage actually dealt, so this is less than the 154.7 exp from 116 damage.
const HP_EXP_PER_HELLHOUND: f64 = 116.3;
/// Hellhounds always drop vile ashes, which give 25 prayer exp when scattered
const PRAYER_EXP_PER_VILE_ASHES: f64 = 25.0;
/// Smouldering stone drop rate from level 122 hellhounds
const SMOULDERING_STONE_CHANCE: f64 = 1.0 / 32_768.0;

fn get_exp(metrics: &[Metric], name: &str) -> eyre::Result<u32> {
    metrics
        .iter()
        .find(|m| m.name == name)
        .with_context(|| format!("{name} metric not found"))?
        .exp
        .with_context(|| format!("No {name} exp found"))
}

pub fn predict_chunkroll_date(metrics: &[Metric]) -> eyre::Result<PredictionResult> {
    predict_chunkroll_date_at(metrics, Utc::now())
}

fn predict_chunkroll_date_at(
    metrics: &[Metric],
    now: DateTime<Utc>,
) -> eyre::Result<PredictionResult> {
    let hp_exp_gained = get_exp(metrics, "Hitpoints")?.saturating_sub(START_HP_EXP);
    let prayer_exp_gained = get_exp(metrics, "Prayer")?.saturating_sub(START_PRAYER_EXP);
    let herblore_exp_left = TARGET_HERBLORE_EXP.saturating_sub(get_exp(metrics, "Herblore")?);

    let hellhounds_kc = hp_exp_gained as f64 / HP_EXP_PER_HELLHOUND;
    let ashes_scattered = prayer_exp_gained as f64 / PRAYER_EXP_PER_VILE_ASHES;

    let grind_start = DateTime::parse_from_rfc3339(GRIND_START)?.to_utc();
    let days_elapsed = (now - grind_start).as_seconds_f64() / 86_400.0;
    eyre::ensure!(
        hellhounds_kc > 0.0 && days_elapsed > 0.0,
        "No hellhound kills since {GRIND_START}"
    );
    let hellhounds_per_day = hellhounds_kc / days_elapsed;

    // The chunkroll happens when he has both 99 herblore and the smouldering stone, so the time
    // left is max(herblore_days, stone_days). The herblore time is assumed to be fixed. The number
    // of kills until the stone is geometric, which is approximated as an exponential distribution
    // of days. It is memoryless, so the kills he has already done don't change it.
    let herblore_days = herblore_exp_left as f64 / HERBLORE_EXP_PER_DAY;
    let mean_stone_days = 1.0 / (SMOULDERING_STONE_CHANCE * hellhounds_per_day);

    // E[max(h, S)] = h + E[max(S - h, 0)] = h + P(S > h) * E[S], by memorylessness
    let average_days = herblore_days + (-herblore_days / mean_stone_days).exp() * mean_stone_days;
    // The quantiles of max(h, S) are the quantiles of S, but at least h
    let quantile_days = |q: f64| herblore_days.max(-mean_stone_days * (1.0 - q).ln());

    let date_in_days = |days: f64| {
        TimeDelta::try_seconds((days * 86_400.0).round() as i64)
            .and_then(|delta| now.checked_add_signed(delta))
            .context("Chunkroll date out of range")
    };

    Ok(PredictionResult {
        herblore_exp_left,
        hellhounds_kc: hellhounds_kc.round() as u32,
        percent_ashes_scattered: (100.0 * ashes_scattered / hellhounds_kc) as f32,
        average_chunkroll_date: date_in_days(average_days)?,
        lower_bound_chunkroll_date: date_in_days(quantile_days(0.025))?,
        upper_bound_chunkroll_date: date_in_days(quantile_days(0.975))?,
    })
}

pub struct PredictionResult {
    pub herblore_exp_left: u32,
    pub hellhounds_kc: u32,
    pub percent_ashes_scattered: f32,
    pub average_chunkroll_date: DateTime<Utc>,
    pub lower_bound_chunkroll_date: DateTime<Utc>,
    pub upper_bound_chunkroll_date: DateTime<Utc>,
}

impl PredictionResult {
    pub fn update_message(&self) -> String {
        format!(
            "Limpwurt still needs another {}k herblore exp for level 99 herblore, and a smouldering stone from hellhounds. He has killed {} hellhounds so far, and scattered {:.1}% of vile ashes. Chunkroll is estimated on **{}**, or between **{}** and **{}** with 95% confidence.",
            self.herblore_exp_left / 1000,
            self.hellhounds_kc,
            self.percent_ashes_scattered,
            self.average_chunkroll_date.format("%d %B %Y"),
            self.lower_bound_chunkroll_date.format("%d %B %Y"),
            self.upper_bound_chunkroll_date.format("%d %B %Y")
        )
    }
}

#[test]
fn test_predict_chunkroll_date() {
    let metric = |name: &str, exp| Metric {
        name: name.to_string(),
        rank: None,
        score: 99,
        exp: Some(exp),
    };
    // Hiscores on 8 November 2026, 10:07 UTC
    let metrics = [
        metric("Hitpoints", 76_465_029),
        metric("Prayer", 4_709_387),
        metric("Herblore", 12_201_400),
    ];
    let now = DateTime::parse_from_rfc3339("2026-11-08T10:07:20Z")
        .unwrap()
        .to_utc();
    let prediction = predict_chunkroll_date_at(&metrics, now).unwrap();
    assert_eq!(
        prediction.update_message(),
        "Limpwurt still needs another 833k herblore exp for level 99 herblore, and a smouldering stone from hellhounds. He has killed 7666 hellhounds so far, and scattered 98.3% of vile ashes. Chunkroll is estimated on **23 December 2026**, or between **16 November 2026** and **22 April 2027** with 95% confidence."
    );
}
