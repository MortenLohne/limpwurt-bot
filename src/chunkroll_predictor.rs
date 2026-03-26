use std::sync::{Arc, Mutex};

use eyre::ContextCompat;

use crate::{db, hiscore_lookup::Metric};

const LEVEL_99: u32 = 13_034_431;

#[derive(Debug, Clone, Copy)]
struct LimpwurtState {
    rc_exp: u32,
    clog_slots: u32,
    pure_essence: u32,
    pages: u32,
    tiaras: u32,
}

impl LimpwurtState {
    fn days_left(&self) -> f32 {
        let rc_exp_left = LEVEL_99.saturating_sub(self.rc_exp);
        let exp_left_after_pages = rc_exp_left.saturating_sub(self.pages * 50);
        let non_banked_exp_left = exp_left_after_pages
            .saturating_sub((self.pure_essence as f32 * 9.5) as u32)
            .saturating_sub(self.tiaras * 25);
        let clog_slots_needed = 299u32.saturating_sub(self.clog_slots);

        let current_ess_rc_days = self.pure_essence as f32 / 25_200.0; // Can use 25.2k pure essence/day
        let tiaras_days = self.tiaras as f32 / 5_600.0; // Can use 5.6k tiaras/day

        let titans_days = non_banked_exp_left as f32 / 58_000.0;

        let days_left =
            current_ess_rc_days + tiaras_days + titans_days + clog_slots_needed as f32 / 0.5;
        println!(
            "Current essence rc days: {}, titans days: {}, tiaras days: {}",
            current_ess_rc_days, titans_days, tiaras_days
        );
        #[allow(clippy::let_and_return)]
        days_left
    }
}

#[derive(Debug, Clone, Copy)]
struct Exp {
    _ranged: u32,
    _hitpoints: u32,
    prayer: u32,
    runecrafting: u32,
    crafting: u32,
    clog_slots: u32,
    brutus_kills: u32,
    titans_kills: u32,
}

impl TryFrom<Vec<Metric>> for Exp {
    type Error = eyre::Error;

    fn try_from(metrics: Vec<Metric>) -> Result<Self, Self::Error> {
        Ok(Exp {
            _ranged: metrics
                .iter()
                .find(|m| m.name == "Ranged")
                .context("Ranged metric not found")?
                .exp
                .context("No ranged exp found")?,
            _hitpoints: metrics
                .iter()
                .find(|m| m.name == "Hitpoints")
                .context("Hitpoints metric not found")?
                .exp
                .context("No hitpoints exp found")?,
            prayer: metrics
                .iter()
                .find(|m| m.name == "Prayer")
                .context("Prayer metric not found")?
                .exp
                .context("No prayer exp found")?,
            runecrafting: metrics
                .iter()
                .find(|m| m.name == "Runecraft")
                .context("Runecrafting metric not found")?
                .exp
                .context("No runecrafting exp found")?,
            crafting: metrics
                .iter()
                .find(|m| m.name == "Crafting")
                .context("Crafting metric not found")?
                .exp
                .context("No crafting exp found")?,
            clog_slots: metrics
                .iter()
                .find(|m| m.name == "Collections Logged")
                .context("Collections Logged metric not found")?
                .score,
            brutus_kills: metrics
                .iter()
                .find(|m| m.name == "Brutus")
                .context("Brutus metric not found")?
                .score,
            titans_kills: metrics
                .iter()
                .find(|m| m.name == "The Royal Titans")
                .context("Royal Titans metric not found")?
                .score,
        })
    }
}

pub struct PredictionResult {
    pub rc_exp_left: u32,
    pub current_pure_essence: u32,
    pub current_pages: u32,
    pub total_pure_essence_needed: u32,
    pub current_tiars: u32,
    pub clog_slots_left: u32,
    pub days_left: f32,
}

/// Predict when the next chunk roll will be, based on the most recent hiscores metrics
pub fn predict_chunkroll_date(
    conn: Arc<Mutex<rusqlite::Connection>>,
) -> eyre::Result<PredictionResult> {
    // November 6th 2025
    // let video_date: chrono::NaiveDate = chrono::NaiveDate::from_ymd_opt(2025, 11, 6).unwrap();
    // println!("Video date: {}", video_date);

    // January 7th 2026
    // let out_of_essence_estimate_date: chrono::NaiveDate =
    //     chrono::NaiveDate::from_ymd_opt(2026, 1, 7).unwrap();
    // let out_of_essence_estimate_exp: Exp = Exp {
    //     ranged: 67_140_006,
    //     hitpoints: 49_370_977,
    //     prayer: 3_449_329,
    //     runecrafting: 9_192_884,
    //     clog_slots: 291,
    //     brutus_kills: 0,
    // };

    // let february_25th_state: Exp = Exp {
    //     _ranged: 76_280_441,
    //     _hitpoints: 53_787_837,
    //     prayer: 4_316_151,
    //     runecrafting: 9_211_312,
    //     crafting: 7_942_745,
    //     clog_slots: 295,
    //     brutus_kills: 89,
    //     titans_kills: 1984,
    // };

    let march_17th_state: Exp = Exp {
        _ranged: 77_628_000,
        _hitpoints: 57_008_000,
        prayer: 4_488_000,
        runecrafting: 9_926_000,
        crafting: 7_942_745,
        clog_slots: 298,
        brutus_kills: 3759,
        titans_kills: 2972,
    };

    // let limpwurt_january_state = LimpwurtState {
    //     rc_exp: out_of_essence_estimate_exp.runecrafting,
    //     clog_slots: out_of_essence_estimate_exp.clog_slots,
    //     pure_essence: 0,
    // };

    // let limpwurt_february_25th_state = LimpwurtState {
    //     rc_exp: february_25th_state.runecrafting,
    //     clog_slots: february_25th_state.clog_slots,
    //     pure_essence: 238_000,
    //     pages: 0,
    // };

    let limpwurt_march_17th_state = LimpwurtState {
        rc_exp: march_17th_state.runecrafting,
        clog_slots: march_17th_state.clog_slots,
        pure_essence: 274_500,
        pages: 0,
        tiaras: 0,
    };

    let metrics = {
        let conn_guard = conn.lock().unwrap();
        db::last_scores(&conn_guard, "OneChunkUp")?
    };
    let current_exp = Exp::try_from(metrics)?;
    let prayer_exp_gained = current_exp.prayer - march_17th_state.prayer;
    let brutus_kills_gained = current_exp.brutus_kills - march_17th_state.brutus_kills;
    // Assume each wyvern gives 62.6 prayer exp, because he banks 15% of the bones
    let wyverns_killed = (prayer_exp_gained.saturating_sub(brutus_kills_gained * 10)) as f32 / 62.6;

    let essence_gained = wyverns_killed * 250.0 / 16.0;
    let pages_gained = (current_exp.titans_kills - march_17th_state.titans_kills) as f32 * 14.5;

    let rc_exp_gained = current_exp.runecrafting - march_17th_state.runecrafting;
    let pages_used =
        (rc_exp_gained / 50).min(pages_gained as u32 + limpwurt_march_17th_state.pages);

    let rc_exp_from_essence_or_tiaras = (rc_exp_gained - pages_used as u32 * 50) as f32;
    let rc_exp_from_essence = rc_exp_from_essence_or_tiaras
        .min((limpwurt_march_17th_state.pure_essence + essence_gained as u32) as f32 * 9.5);
    let rc_exp_from_tiaras = rc_exp_from_essence_or_tiaras - rc_exp_from_essence;

    println!(
        "Gained {}k rc exp, used {} pages, got {:.1}k rc exp from essence or tiaras, {:.1}k from essence, {:.1}k from tiaras",
        rc_exp_gained / 1000,
        pages_used,
        rc_exp_from_essence_or_tiaras / 1000.0,
        rc_exp_from_essence / 1000.0,
        rc_exp_from_tiaras / 1000.0,
    );
    let essence_used = (rc_exp_from_essence as f32 / 9.5) as u32;
    let current_essence = (limpwurt_march_17th_state.pure_essence + essence_gained as u32)
        .checked_sub(essence_used)
        .unwrap_or_else(|| {
            println!(
                "Warning: Pure essence int overflow. Has essence: {}, essence_used: {}",
                (limpwurt_march_17th_state.pure_essence + essence_gained as u32),
                essence_used
            );
            0
        });

    let current_pages = limpwurt_march_17th_state.pages + pages_gained as u32 - pages_used;

    let tiaras_made =
        ((current_exp.crafting - march_17th_state.crafting) as f32 / 52.5).round() as u32;
    let tiaras_used = (rc_exp_from_tiaras as f32 / 25.0).round() as u32;
    let current_tiars = limpwurt_march_17th_state.tiaras + tiaras_made - tiaras_used;

    let limpwurt_state = LimpwurtState {
        rc_exp: current_exp.runecrafting,
        clog_slots: current_exp.clog_slots,
        pure_essence: current_essence,
        pages: current_pages,
        tiaras: current_tiars,
    };

    let days_left = limpwurt_state.days_left();

    Ok(PredictionResult {
        rc_exp_left: LEVEL_99.saturating_sub(current_exp.runecrafting),
        current_pure_essence: current_essence,
        current_pages: current_pages as u32,
        current_tiars: current_tiars,
        total_pure_essence_needed: (LEVEL_99.saturating_sub(current_exp.runecrafting) as f32 / 9.5)
            as u32,
        clog_slots_left: 299u32.saturating_sub(current_exp.clog_slots),
        days_left,
    })
}
