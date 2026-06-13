use crate::matcher::Matcher;
use crate::models::{EngineRequest, Order, OrderType, Side, TradingPair};
use crate::orderbook::OrderBook;
use crate::redis::RedisClient;
use serde_json::json;

pub fn handle_request(request: EngineRequest, orderbook: &mut OrderBook, redis: &mut RedisClient) {
    match request.r#type.as_str() {
        "place_order" => handle_place_order(request, orderbook, redis),
        "cancel_order" => handle_cancel_order(request, orderbook, redis),
        "get_depth" => handle_get_depth(request, orderbook, redis),
        "get_ticker" => handle_get_ticker(request, orderbook, redis),
        _ => {
            redis.send_error(
                &request.correlation_id,
                &request.response_queue,
                &format!("unknown command: {}", request.r#type),
            );
        }
    }
}

fn handle_place_order(request: EngineRequest, orderbook: &mut OrderBook, redis: &mut RedisClient) {
    let payload = &request.payload;

    let user_id = match payload["userId"].as_str() {
        Some(v) => v.to_string(),
        None => {
            redis.send_error(
                &request.correlation_id,
                &request.response_queue,
                "missing userId",
            );
            return;
        }
    };

    let symbol = match payload["symbol"].as_str() {
        Some(v) => v.to_string(),
        None => {
            redis.send_error(
                &request.correlation_id,
                &request.response_queue,
                "missing symbol",
            );
            return;
        }
    };

    let side = match payload["side"].as_str() {
        Some("buy") => Side::Buy,
        Some("sell") => Side::Sell,
        _ => {
            redis.send_error(
                &request.correlation_id,
                &request.response_queue,
                "invalid side",
            );
            return;
        }
    };

    let order_type = match payload["orderType"].as_str() {
        Some("limit") => OrderType::Limit,
        Some("market") => OrderType::Market,
        _ => {
            redis.send_error(
                &request.correlation_id,
                &request.response_queue,
                "invalid orderType",
            );
            return;
        }
    };

    let price = payload["price"].as_f64().unwrap_or(0.0);

    let quantity = match payload["quantity"].as_f64() {
        Some(v) => v,
        None => {
            redis.send_error(
                &request.correlation_id,
                &request.response_queue,
                "missing quantity",
            );
            return;
        }
    };

    let trading_pair = match parse_trading_pair(&symbol) {
        Some(p) => p,
        None => {
            redis.send_error(
                &request.correlation_id,
                &request.response_queue,
                "invalid symbol",
            );
            return;
        }
    };

    let mut order = Order::new(user_id, trading_pair, side, order_type, price, quantity);
    let trades = Matcher::process(&mut order, orderbook);

    // publish updated orderbook to pub/sub
    let depth = orderbook.get_depth(20);
    redis.publish(&format!("orderbook:{}", symbol), json!(depth));

    // publish trades if any
    if !trades.is_empty() {
        redis.publish(&format!("trades:{}", symbol), json!(trades));
    }

    redis.send_success(
        &request.correlation_id,
        &request.response_queue,
        json!({
            "orderId": order.id,
            "status": format!("{:?}", order.status),
            "filled": order.filled,
            "remaining": order.remaining(),
            "trades": trades,
        }),
    );
}

fn handle_cancel_order(request: EngineRequest, orderbook: &mut OrderBook, redis: &mut RedisClient) {
    let order_id = match request.payload["orderId"].as_str() {
        Some(v) => v,
        None => {
            redis.send_error(
                &request.correlation_id,
                &request.response_queue,
                "missing orderId",
            );
            return;
        }
    };

    match orderbook.cancel_order(order_id) {
        Some(order) => {
            redis.send_success(
                &request.correlation_id,
                &request.response_queue,
                json!({
                    "orderId": order.id,
                    "status": "cancelled",
                }),
            );
        }
        None => {
            redis.send_error(
                &request.correlation_id,
                &request.response_queue,
                "order not found",
            );
        }
    }
}

fn handle_get_depth(request: EngineRequest, orderbook: &mut OrderBook, redis: &mut RedisClient) {
    let levels = request.payload["levels"].as_f64().unwrap_or(20.0) as usize;

    let depth = orderbook.get_depth(levels);

    redis.send_success(
        &request.correlation_id,
        &request.response_queue,
        json!(depth),
    );
}

fn handle_get_ticker(request: EngineRequest, orderbook: &mut OrderBook, redis: &mut RedisClient) {
    redis.send_success(
        &request.correlation_id,
        &request.response_queue,
        json!({
            "bestBid": orderbook.best_bid(),
            "bestAsk": orderbook.best_ask(),
            "spread": match (orderbook.best_bid(), orderbook.best_ask()) {
                (Some(bid), Some(ask)) => Some(ask - bid),
                _ => None,
            }
        }),
    );
}

fn parse_trading_pair(symbol: &str) -> Option<TradingPair> {
    match symbol {
        "BTC_USDT" => Some(TradingPair::BTC_USDT),
        "ETH_USDT" => Some(TradingPair::ETH_USDT),
        "SOL_USDT" => Some(TradingPair::SOL_USDT),
        _ => None,
    }
}
