use limpwurt_chunk_roll::dagan_hai_simulator::simulate_muddy_chests;
use rand::SeedableRng;
use std::collections::BTreeMap;

pub fn main() {
    let mut rng = rand::rngs::SmallRng::from_rng(&mut rand::rng());
    let mut grind_length_histogram: BTreeMap<u32, u64> = BTreeMap::new();
    let mut grind_lengths: Vec<u32> = Vec::new();
    for _ in 0..100_000 {
        let (_, grind_length) = simulate_muddy_chests(&mut rng, 3);
        *grind_length_histogram.entry(grind_length).or_insert(0) += 1;
        grind_lengths.push(grind_length);
    }

    grind_lengths.sort();

    println!(
        "Median grind length: {}",
        grind_lengths[grind_lengths.len() / 2]
    );
    println!(
        "Average grind length: {}",
        grind_lengths.iter().map(|l| *l as u64).sum::<u64>() / grind_lengths.len() as u64
    );
    println!(
        "2.5% grind length: {}",
        grind_lengths[grind_lengths.len() / 40]
    );
    println!(
        "97.5% grind length: {}",
        grind_lengths[grind_lengths.len() * 39 / 40]
    );
}
