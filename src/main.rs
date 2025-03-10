mod alpha_beta;
mod simulate_game;
mod ai;
mod cheater;

use std::time::Instant;

fn main() {

    // run some games and measure the execution time
    let iterations = 10;
    let search_depth = 6;
    let now = Instant::now();
    for _ in 0..iterations {
        simulate_game::four_cheaters(search_depth);
    }
    let elapsed = now.elapsed();
    let average = elapsed.div_f64(f64::from(iterations));

    // print the execution time and some information about the calculated search trees
    println!("\n ---------- Test results ----------");
    println!("number of simulated games: {}", iterations);
    println!("total elapsed time: {:?}", elapsed);
    println!("avg execution time per game: {:?}", average);
    unsafe {
        cheater::print_avg_tree_size();
    }
}