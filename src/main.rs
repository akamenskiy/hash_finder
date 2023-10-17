use std::time::Instant;
use hash_finder;

fn main() {
    let args: hash_finder::Arguments = argh::from_env();
    let start = Instant::now();

    hash_finder::run(args);

    let duration = start.elapsed();
    println!("Time elapsed in expensive_function() is: {:?}", duration);
}