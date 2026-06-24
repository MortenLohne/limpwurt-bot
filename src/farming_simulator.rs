use std::fmt;

enum Scenario {
    Herbs1,
    Herbs2,
    Herbs3,
    Other,
}

enum HerbRunResult {
    Survived(Vec<(Herb, usize)>),
    Died,
}

impl Scenario {
    /// Returns potential herb run to get this amount of exp
    pub fn match_against_exp(self, exp: Exp) -> Vec<HerbRunResult> {
        match self {
            Scenario::Herbs1 => {
                todo!()
            }
            _ => todo!(),
        }
    }
}

/// Experience times 10
pub struct Exp(u32);

impl fmt::Display for Exp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.0 / 10, self.0 % 10)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Compost {
    NoCompost,
    Compost,
    Supercompost,
    Ultracompost,
}

impl Compost {
    pub fn death_chance(self) -> f32 {
        1.0 - (1.0f32
            - match self {
                Compost::NoCompost => 27.0 / 128.0,
                Compost::Compost => 14.0 / 128.0,
                Compost::Supercompost => 6.0 / 128.0,
                Compost::Ultracompost => 3.0 / 128.0,
            })
        .powf(3.0)
    }

    pub fn lives(self) -> u32 {
        match self {
            Compost::NoCompost => 3,
            Compost::Compost => 4,
            Compost::Supercompost => 5,
            Compost::Ultracompost => 6,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Herb {
    GuamLeaf,
    Marrentill,
    Tarromin,
    Harralander,
    Ranarr,
    Toadflax,
    IritLeaf,
    Avantoe,
    Kwuarm,
    Snapdragon,
    Cadantine,
    Lantadyme,
    DwarfWeed,
    Torstol,
}

use Herb::*;
use rand::{Rng, RngExt};

impl Herb {
    pub fn plant_exp(self) -> Exp {
        match self {
            GuamLeaf => Exp(110),
            Marrentill => Exp(135),
            Tarromin => Exp(160),
            Harralander => Exp(215),
            Ranarr => Exp(270),
            Toadflax => Exp(340),
            IritLeaf => Exp(430),
            Avantoe => Exp(545),
            Kwuarm => Exp(690),
            Snapdragon => Exp(875),
            Cadantine => Exp(1065),
            Lantadyme => Exp(1345),
            DwarfWeed => Exp(1705),
            Torstol => Exp(1995),
        }
    }

    pub fn harvest_exp(self) -> Exp {
        match self {
            GuamLeaf => Exp(125),
            Marrentill => Exp(150),
            Tarromin => Exp(180),
            Harralander => Exp(240),
            Ranarr => Exp(305),
            Toadflax => Exp(385),
            IritLeaf => Exp(485),
            Avantoe => Exp(615),
            Kwuarm => Exp(780),
            Snapdragon => Exp(985),
            Cadantine => Exp(1200),
            Lantadyme => Exp(1515),
            DwarfWeed => Exp(1920),
            Torstol => Exp(2245),
        }
    }

    pub fn low_cts(self) -> f32 {
        match self {
            GuamLeaf => 25.0,
            Marrentill => 28.0,
            Tarromin => 31.0,
            Harralander => 36.0,
            Ranarr => 39.0,
            Toadflax => 43.0,
            IritLeaf => 46.0,
            Avantoe => 50.0,
            Kwuarm => 54.0,
            Snapdragon => 57.0,
            Cadantine => 60.0,
            Lantadyme => 64.0,
            DwarfWeed => 67.0,
            Torstol => 71.0,
        }
    }

    pub fn high_cts(self) -> f32 {
        80.0
    }
}

pub fn save_life_chance(
    farming_level: u32,
    farming_cape: bool,
    magic_sec: bool,
    herb: Herb,
) -> f32 {
    let cts_base_low = herb.low_cts();
    let cts_base_high = herb.high_cts();

    let cts_low = (cts_base_low
        * (1.0 + if magic_sec { 0.1 } else { 0.0 } + if farming_cape { 0.05 } else { 0.0 }))
    .floor();

    let cts_high = (cts_base_high
        * (1.0 + if magic_sec { 0.1 } else { 0.0 } + if farming_cape { 0.05 } else { 0.0 }))
    .floor();

    let numerator = (cts_low * (99.0 - farming_level as f32) / 98.0)
        + (cts_high * (farming_level as f32 - 1.0) / 98.0)
        + 0.5;
    let chance = (1.0 + numerator.floor()) / 256.0;
    assert!(chance > 0.0);
    assert!(chance < 1.0);
    chance
}

pub fn simulate_herb_run<R: Rng>(
    rng: &mut R,
    farming_level: u32,
    farming_cape: bool,
    magic_sec: bool,
    compost: Compost,
    herb: Herb,
) -> u32 {
    if rng.random_bool(compost.death_chance() as f64) {
        return 0;
    }
    let p = save_life_chance(farming_level, farming_cape, magic_sec, herb);
    let mut lives = compost.lives();
    let mut herbs = 0;
    while lives > 0 {
        herbs += 1;
        if !rng.random_bool(p as f64) {
            lives -= 1;
        }
    }
    herbs
}

pub fn choose_weighted<'a, R: Rng, T>(rng: &mut R, items: &'a [(T, u32)]) -> &'a T {
    let total_weight: u32 = items.iter().map(|(_, weight)| *weight).sum();
    let mut cumulative_weight: u32 = 0;
    for (item, weight) in items {
        cumulative_weight += *weight;
        if rng.random::<u32>() % total_weight < cumulative_weight {
            return &item;
        }
    }
    unreachable!()
}

#[test]
fn choose_weighted_test() {
    let items = &[("apple", 10), ("banana", 20), ("cherry", 30)];
    for _ in 0..10 {
        let result = choose_weighted(&mut rand::rng(), items);
        assert!(items.iter().any(|(name, _)| name == result));
    }
}
