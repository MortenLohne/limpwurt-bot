use std::collections::BTreeMap;

use rand::Rng;

use crate::drop_simulator::{self, Item};

/// Returns the drops and number of Larran's chests needed to obtain the full Dagon Hai set.
pub fn simulate_larran_chests<R: Rng>(
    rng: &mut R,
    starting_uniques: u32,
) -> (BTreeMap<Item, u32>, u32) {
    let mut drops: BTreeMap<Item, u32> = BTreeMap::new();
    if starting_uniques > 0 {
        drops.insert(Item::DagonHaiHat, 1);
    }
    if starting_uniques > 1 {
        drops.insert(Item::DagonHaiTop, 1);
    }
    if starting_uniques > 2 {
        drops.insert(Item::DagonHaiBottom, 1);
    }

    let mut num_chests = 0;

    while !(drops.contains_key(&Item::DagonHaiHat)
        && drops.contains_key(&Item::DagonHaiTop)
        && drops.contains_key(&Item::DagonHaiBottom))
    {
        num_chests += 1;
        for (item, quantity) in drop_simulator::LARRANS_CHEST.roll(rng) {
            drops
                .entry(item)
                .and_modify(|q| *q += quantity)
                .or_insert(quantity);
        }
    }

    (drops, num_chests)
}

/// Returns the drops and number of Muddy's chests needed to obtain the full Dagon Hai set.
pub fn simulate_muddy_chests<R: Rng>(
    rng: &mut R,
    starting_uniques: u32,
) -> (BTreeMap<Item, u32>, u32) {
    let mut drops: BTreeMap<Item, u32> = BTreeMap::new();
    if starting_uniques > 0 {
        drops.insert(Item::DagonHaiHat, 1);
    }
    if starting_uniques > 1 {
        drops.insert(Item::DagonHaiTop, 1);
    }
    if starting_uniques > 2 {
        drops.insert(Item::DagonHaiBottom, 1);
    }

    let mut num_chests = 0;

    while !(drops.contains_key(&Item::DagonHaiHat)
        && drops.contains_key(&Item::DagonHaiTop)
        && drops.contains_key(&Item::DagonHaiBottom))
    {
        num_chests += 1;
        for (item, quantity) in drop_simulator::MUDDY_CHEST.roll(rng) {
            drops
                .entry(item)
                .and_modify(|q| *q += quantity)
                .or_insert(quantity);
            // Also roll Larrans chest if we get a Larrans key
            if item == Item::LarransKey {
                for (item, quantity) in drop_simulator::LARRANS_CHEST.roll(rng) {
                    drops
                        .entry(item)
                        .and_modify(|q| *q += quantity)
                        .or_insert(quantity);
                }
            }
        }
    }

    (drops, num_chests)
}
