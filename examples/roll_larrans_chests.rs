use limpwurt_chunk_roll::drop_simulator::Item;
use limpwurt_chunk_roll::drop_simulator::LARRANS_CHEST;
use std::collections::BTreeMap;

pub fn main() {
    let mut rng = rand::rng();
    let mut drops: BTreeMap<Item, u64> = BTreeMap::new();
    for _ in 0..1_000_000 {
        let drop = LARRANS_CHEST.roll(&mut rng);
        for (item, quantity) in drop {
            *drops.entry(item).or_insert(0) += quantity as u64;
        }
    }
    for (item, quantity) in drops {
        println!("{:?}: {}", item, quantity);
    }
}
