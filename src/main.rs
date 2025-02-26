mod alpha_beta;
mod simulate_game;
mod ai;
mod cheater;

use std::time::Instant;

fn main() {
    let iterations = 1;
    let now = Instant::now();
    for _ in 0..iterations {
        simulate_game::four_cheaters();
    }
    let elapsed = now.elapsed();
    let average = elapsed.div_f64(f64::from(iterations));
    println!("\ntotal elapsed time: {:?}", elapsed);
    println!("avg execution time: {:?}", average);
}