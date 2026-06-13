mod models;
mod orderbook;
mod matcher;
mod redis;
mod handlers;
mod error;

use std::thread;
use std::sync::mpsc;
use std::collections::HashMap;
use models::{EngineRequest, EngineState, TradingPair};
use orderbook::OrderBook;
use redis::RedisClient;
use handlers::handle_request;
use strum::IntoEnumIterator;

fn main() {
    dotenvy::dotenv().ok();
    println!("Starting CEX engine...");

    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let incoming_queue = std::env::var("INCOMING_QUEUE")
        .unwrap_or_else(|_| "backend-to-engine-broker".to_string());

    let mut senders: HashMap<String, mpsc::Sender<EngineRequest>> = HashMap::new();

    for pair in TradingPair::iter() {
        let (tx, rx) = mpsc::channel::<EngineRequest>();
        let redis_url = redis_url.clone();

        senders.insert(pair.to_string(), tx);

        thread::spawn(move || {
            println!("Matching engine ready for {}", pair);

            let mut orderbook = OrderBook::new(pair.clone());
            let mut state = EngineState::new();
            let mut redis = RedisClient::new(&redis_url);

            while let Ok(request) = rx.recv() {
                handle_request(request, &mut orderbook, &mut state, &mut redis);
            }
        });
    }

    let reader = thread::spawn(move || {
        let mut redis = RedisClient::new(&redis_url);

        println!("Listening on queue: {}", incoming_queue);

        loop {
            match redis.pop_request(&incoming_queue) {
                Some(request) => {
                    let symbol = request.payload["symbol"]
                        .as_str()
                        .unwrap_or("")
                        .to_string();

                    match senders.get(&symbol) {
                        Some(tx) => {
                            if let Err(e) = tx.send(request) {
                                eprintln!("Failed to route request for {}: {}", symbol, e);
                            }
                        }
                        None => {
                            eprintln!("Unknown symbol: {}", symbol);
                        }
                    }
                }
                None => continue,
            }
        }
    });

    reader.join().unwrap();
}