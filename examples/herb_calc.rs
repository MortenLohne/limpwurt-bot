use std::collections::BTreeMap;

use limpwurt_chunk_roll::farming_simulator::{self, Compost, Herb};

fn main() {
    let mut rng = rand::rng();
    let n = 1_000_000_000;

    let mut total: u64 = 0;
    let mut buckets: BTreeMap<u32, u64> = BTreeMap::new();
    for _ in 0..n {
        let result = farming_simulator::simulate_herb_run(
            &mut rng,
            40,
            false,
            true,
            Compost::Supercompost,
            Herb::Ranarr,
        );
        total += result as u64;
        *buckets.entry(result).or_default() += 1;
    }
    println!("Average: {:.3}", total as f64 / n as f64);
    println!("Buckets:");
    for (k, v) in &buckets {
        println!("{:3}: {:9}  {:.4}%", k, v, *v as f64 / n as f64 * 100.0);
    }
}
