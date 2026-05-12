use arrayvec::ArrayVec;
use rand::{Rng, RngExt};

pub const MUDDY_CHEST: DropTable = DropTable {
    guaranteed: &[
        (Item::UncutRuby, 1),
        (Item::MithrilBar, 2),
        (Item::LawRune, 5),
        (Item::DeathRune, 5),
        (Item::ChaosRune, 15),
    ],
    pre_roll: &[],
    main_roll: &[
        DropTableEntry {
            item: Some(Item::BlightedMantaRay),
            min_amount: 25,
            max_amount: 25,
            weight: 3,
        },
        DropTableEntry {
            item: Some(Item::BlightedKarambwan),
            min_amount: 25,
            max_amount: 25,
            weight: 3,
        },
        DropTableEntry {
            item: Some(Item::BlightedAncientIceSack),
            min_amount: 25,
            max_amount: 25,
            weight: 3,
        },
        DropTableEntry {
            item: Some(Item::BlightedAnglerfish),
            min_amount: 15,
            max_amount: 15,
            weight: 3,
        },
        DropTableEntry {
            item: Some(Item::BlightedSuperRestore),
            min_amount: 3,
            max_amount: 3,
            weight: 3,
        },
        DropTableEntry {
            item: Some(Item::LarransKey),
            min_amount: 1,
            max_amount: 1,
            weight: 6,
        },
        DropTableEntry {
            item: None,
            min_amount: 0,
            max_amount: 0,
            weight: 259,
        },
    ],
};

pub const LARRANS_CHEST: DropTable = DropTable {
    guaranteed: &[],
    pre_roll: &[
        DropTableEntry {
            item: Some(Item::DagonHaiHat),
            min_amount: 1,
            max_amount: 1,
            weight: 1,
        },
        DropTableEntry {
            item: Some(Item::DagonHaiTop),
            min_amount: 1,
            max_amount: 1,
            weight: 1,
        },
        DropTableEntry {
            item: Some(Item::DagonHaiBottom),
            min_amount: 1,
            max_amount: 1,
            weight: 1,
        },
        DropTableEntry {
            item: None,
            min_amount: 0,
            max_amount: 0,
            weight: 253,
        },
    ],
    main_roll: &[
        DropTableEntry {
            item: Some(Item::UncutDiamond),
            min_amount: 35,
            max_amount: 45,
            weight: 5,
        },
        DropTableEntry {
            item: Some(Item::UncutRuby),
            min_amount: 35,
            max_amount: 45,
            weight: 5,
        },
        DropTableEntry {
            item: Some(Item::Coal),
            min_amount: 450,
            max_amount: 650,
            weight: 5,
        },
        DropTableEntry {
            item: Some(Item::GoldOre),
            min_amount: 150,
            max_amount: 250,
            weight: 4,
        },
        DropTableEntry {
            item: Some(Item::DragonArrowtips),
            min_amount: 100,
            max_amount: 250,
            weight: 4,
        },
        DropTableEntry {
            item: Some(Item::Coins),
            min_amount: 75000,
            max_amount: 175000,
            weight: 3,
        },
        DropTableEntry {
            item: Some(Item::IronOre),
            min_amount: 500,
            max_amount: 650,
            weight: 3,
        },
        DropTableEntry {
            item: Some(Item::RuneFullHelm),
            min_amount: 3,
            max_amount: 5,
            weight: 3,
        },
        DropTableEntry {
            item: Some(Item::RunePlatebody),
            min_amount: 2,
            max_amount: 3,
            weight: 3,
        },
        DropTableEntry {
            item: Some(Item::RunePlatelegs),
            min_amount: 2,
            max_amount: 3,
            weight: 3,
        },
        DropTableEntry {
            item: Some(Item::PureEssence),
            min_amount: 4500,
            max_amount: 7500,
            weight: 3,
        },
        DropTableEntry {
            item: Some(Item::RuniteOre),
            min_amount: 15,
            max_amount: 20,
            weight: 2,
        },
        DropTableEntry {
            item: Some(Item::SteelBar),
            min_amount: 350,
            max_amount: 550,
            weight: 2,
        },
        DropTableEntry {
            item: Some(Item::MagicLogs),
            min_amount: 180,
            max_amount: 220,
            weight: 2,
        },
        DropTableEntry {
            item: Some(Item::DragonDartTip),
            min_amount: 80,
            max_amount: 200,
            weight: 2,
        },
        DropTableEntry {
            item: Some(Item::PalmTreeSeed),
            min_amount: 3,
            max_amount: 5,
            weight: 1,
        },
        DropTableEntry {
            item: Some(Item::MagicSeed),
            min_amount: 3,
            max_amount: 4,
            weight: 1,
        },
        DropTableEntry {
            item: Some(Item::CelastrusSeed),
            min_amount: 3,
            max_amount: 5,
            weight: 1,
        },
        DropTableEntry {
            item: Some(Item::DragonfruitTreeSeed),
            min_amount: 3,
            max_amount: 5,
            weight: 1,
        },
        DropTableEntry {
            item: Some(Item::RedwoodTreeSeed),
            min_amount: 1,
            max_amount: 1,
            weight: 1,
        },
        DropTableEntry {
            item: Some(Item::TorstolSeed),
            min_amount: 4,
            max_amount: 6,
            weight: 1,
        },
        DropTableEntry {
            item: Some(Item::SnapdragonSeed),
            min_amount: 4,
            max_amount: 6,
            weight: 1,
        },
        DropTableEntry {
            item: Some(Item::RanarrSeed),
            min_amount: 4,
            max_amount: 6,
            weight: 1,
        },
    ],
};

pub struct DropTable {
    guaranteed: &'static [(Item, u32)],
    pre_roll: &'static [DropTableEntry],
    main_roll: &'static [DropTableEntry],
}

impl DropTable {
    pub fn roll<R: Rng>(&self, rng: &mut R) -> ArrayVec<(Item, u32), 10> {
        let mut drops = ArrayVec::from_iter(self.guaranteed.iter().copied());

        let pre_roll_sum = self.pre_roll.iter().map(|e| e.weight).sum::<u32>();
        let pre_roll_roll = rng.random_range(0..pre_roll_sum.max(1));
        let mut sum = 0;
        for entry in self.pre_roll {
            sum += entry.weight;
            if sum > pre_roll_roll {
                if let Some(item) = entry.item {
                    let quantity = rng.random_range(entry.min_amount..=entry.max_amount);
                    drops.push((item, quantity));
                    return drops;
                }
                // If pre-roll entry has no item, roll main table
                break;
            }
        }

        let main_roll_sum = self.main_roll.iter().map(|e| e.weight).sum::<u32>();
        let main_roll_roll = rng.random_range(0..main_roll_sum);
        let mut sum = 0;
        for entry in self.main_roll {
            sum += entry.weight;
            if sum > main_roll_roll {
                if let Some(item) = entry.item {
                    let quantity = rng.random_range(entry.min_amount..=entry.max_amount);
                    drops.push((item, quantity));
                }
                return drops;
            }
        }

        return drops;
    }
}

struct DropTableEntry {
    item: Option<Item>,
    min_amount: u32,
    max_amount: u32,
    weight: u32,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub enum Item {
    DagonHaiHat,
    DagonHaiTop,
    DagonHaiBottom,
    UncutDiamond,
    UncutRuby,
    Coal,
    GoldOre,
    DragonArrowtips,
    Coins,
    IronOre,
    RuneFullHelm,
    RunePlatebody,
    RunePlatelegs,
    PureEssence,
    RuniteOre,
    SteelBar,
    MagicLogs,
    DragonDartTip,
    PalmTreeSeed,
    MagicSeed,
    CelastrusSeed,
    DragonfruitTreeSeed,
    RedwoodTreeSeed,
    TorstolSeed,
    SnapdragonSeed,
    RanarrSeed,
    MuddyKey,
    MithrilBar,
    LawRune,
    DeathRune,
    ChaosRune,
    LarransKey,
    BlightedMantaRay,
    BlightedKarambwan,
    BlightedAncientIceSack,
    BlightedAnglerfish,
    BlightedSuperRestore,
}
