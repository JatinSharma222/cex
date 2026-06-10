use std::thread;
use crate::models::TradingPair;
use strum::IntoEnumIterator;


fn main() {
    // spawn a thread per trading pair

    let handles: Vec<_> = TradingPair::iter().map(|pair| {
        thread::spawn(move || {
            println!("Starting orderbook engine for {} ", pair);
            // each thread has its own orderbook + redis listener
            run_orderbook_engine(pair);
        })
    }).collect();

    // wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
}