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