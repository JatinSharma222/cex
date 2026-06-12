use std::collections::BTreeMap;
use crate::models::{Order, Side, Trade, TradingPair};

#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub price: f64,
    pub orders: Vec<Order>,
}

impl PriceLevel {
    pub fn new(price: f64) -> Self {
        Self {
            price,
            orders: Vec::new(),
        }
    }

    pub fn total_quantity(&self) -> f64 {
        self.orders.iter().map(|o| o.remaining()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct OrderBook {
    pub trading_pair: TradingPair,

    pub bids: BTreeMap<u64, PriceLevel>, // buy orders
    pub asks: BTreeMap<u64, PriceLevel>, // sell orders
}

impl OrderBook {
    pub fn new(trading_pair: TradingPair) -> Self {
        Self {
            trading_pair,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    // convert f64 price to u64 key (multiply by 10^8 to avoid float issues)
    fn price_to_key(price: f64) -> u64 {
        (price * 1_000_000_00.0) as u64
    }

    pub fn add_order(&mut self, order: Order) {
        let key = Self::price_to_key(order.price);
        let book = match order.side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };
        book.entry(key)
            .or_insert_with(|| PriceLevel::new(order.price))
            .orders
            .push(order);
    }

    pub fn cancel_order(&mut self, order_id: &str) -> Option<Order> {
        for book in [&mut self.bids, &mut self.asks] {
            for level in book.values_mut() {
                if let Some(pos) = level.orders.iter().position(|o| o.id == order_id) {
                    return Some(level.orders.remove(pos));
                }
            }
        }
        None
    }

    pub fn best_bid(&self) -> Option<f64> {
        self.bids.keys().next_back().map(|k| *k as f64 / 1_000_000_00.0)
    }

    pub fn best_ask(&self) -> Option<f64> {
        self.asks.keys().next().map(|k| *k as f64 / 1_000_000_00.0)
    }

    // depth snapshot for frontend
    // rerurns (price , total_quantity) for each level
    pub fn get_depth(&self, levels: usize) -> OrderBookDepth {
        let bids: Vec<[f64; 2]> = self.bids
            .iter()
            .rev()
            .take(levels)
            .map(|(_, level)| [level.price, level.total_quantity()])
            .collect();

         let asks: Vec<[f64; 2]> = self.asks
            .iter()  
            .take(levels)
            .map(|(_, level)| [level.price, level.total_quantity()])
            .collect();

        OrderBookDepth { bids, asks }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OrderBookDepth {
    pub bids: Vec<[f64; 2]>, 
    pub asks: Vec<[f64; 2]>,
}