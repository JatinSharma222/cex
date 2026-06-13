
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};
use strum_macros::Display;
use uuid::Uuid;
use std::collections::HashMap;



#[derive(Debug, Clone)]
pub struct EngineState {
    pub orders: HashMap<String, Order>,       
    pub balances: HashMap<String, UserBalance>, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBalance {
    pub user_id: String,
    pub usdt: f64,
    pub btc: f64,
    pub eth: f64,
    pub sol: f64,
}

impl EngineState {
    pub fn new() -> Self {
        Self {
            orders: HashMap::new(),
            balances: HashMap::new(),
        }
    }

    pub fn get_or_create_balance(&mut self, user_id: &str) -> &mut UserBalance {
        self.balances.entry(user_id.to_string()).or_insert(UserBalance {
            user_id: user_id.to_string(),
            usdt: 100_000.0, // starting balance for testing
            btc: 10.0,
            eth: 100.0,
            sol: 1000.0,
        })
    }
}

#[derive(Debug, Clone, EnumIter, Display, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(non_camel_case_types)]  
pub enum TradingPair {
    BTC_USDT,
    ETH_USDT,
    SOL_USDT,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)] 
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]  
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]  
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub trading_pair: TradingPair,
    pub side: Side,
    pub order_type: OrderType,
    pub price: f64,
    pub quantity: f64,
    pub filled: f64,
    pub status: OrderStatus,
}

impl Order {
    pub fn new(
        user_id: String,
        trading_pair: TradingPair,
        side: Side,
        order_type: OrderType,
        price: f64,
        quantity: f64,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            trading_pair,
            side,
            order_type,
            price,
            quantity,
            filled: 0.0,
            status: OrderStatus::Open,
        }
    }

    pub fn remaining(&self) -> f64 {
        self.quantity - self.filled
    }

    pub fn is_filled(&self) -> bool {
        self.filled >= self.quantity
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub trading_pair: TradingPair,
    pub maker_order_id: String,
    pub taker_order_id: String,
    pub price: f64,
    pub quantity: f64,
}

impl Trade {
    pub fn new(
        trading_pair: TradingPair,
        maker_order_id: String,
        taker_order_id: String,
        price: f64,
        quantity: f64,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            trading_pair,
            maker_order_id,
            taker_order_id,
            price,
            quantity,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineRequest {
    pub correlation_id: String,
    pub response_queue: String,  
    pub r#type: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineResponse {
    pub correlation_id: String,
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}