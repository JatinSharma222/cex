use crate::models::{Order, OrderStatus, OrderType, Side, Trade};
use crate::orderbook::OrderBook;

pub struct Matcher;

impl Matcher {
    pub fn process(order: &mut Order, orderbook: &mut OrderBook) -> Vec<Trade> {
        match order.order_type {
            OrderType::Limit => Self::match_limit(order, orderbook),
            OrderType::Market => Self::match_market(order, orderbook),
        }
    }

    fn match_limit(order: &mut Order, orderbook: &mut OrderBook) -> Vec<Trade> {
        let mut trades = Vec::new();

        Self::match_against_book(order, orderbook, &mut trades);

        if !order.is_filled() {
            orderbook.add_order(order.clone());
            order.status = if order.filled > 0.0 {
                OrderStatus::PartiallyFilled
            } else {
                OrderStatus::Open
            };
        } else {
            order.status = OrderStatus::Filled;
        }

        trades
    }

    fn match_market(order: &mut Order, orderbook: &mut OrderBook) -> Vec<Trade> {
        let mut trades = Vec::new();

        Self::match_against_book(order, orderbook, &mut trades);

        order.status = if order.is_filled() {
            OrderStatus::Filled
        } else if order.filled > 0.0 {
            OrderStatus::PartiallyFilled
        } else {
            OrderStatus::Cancelled
        };

        trades
    }

    fn match_against_book(
        order: &mut Order,
        orderbook: &mut OrderBook,
        trades: &mut Vec<Trade>,
    ) {
        let opposite_book = match order.side {
            Side::Buy => &mut orderbook.asks,
            Side::Sell => &mut orderbook.bids,
        };

        let mut keys: Vec<u64> = match order.side {
            Side::Buy => opposite_book.keys().cloned().collect(),
            Side::Sell => opposite_book.keys().cloned().rev().collect(),
        };

        for key in keys {
            if order.is_filled() {
                break;
            }

            let level = match opposite_book.get_mut(&key) {
                Some(l) => l,
                None => continue,
            };

            let price_ok = match order.order_type {
                OrderType::Market => true,
                OrderType::Limit => match order.side {
                    Side::Buy => level.price <= order.price,
                    Side::Sell => level.price >= order.price,
                },
            };

            if !price_ok {
                break;
            }

            let mut filled_indices = Vec::new();

            for(i, maker) in level.orders.iter_mut().enumerate() {
                if order.is_filled() {
                    break;
                }

                let fill_qty = order.remaining().min(maker.remaining());

                order.filled += fill_qty;
                maker.filled += fill_qty;

                maker.status = if maker.is_filled() {
                    filled_indices.push(i);
                    OrderStatus::Filled
                } else {
                    OrderStatus::PartiallyFilled
                };

                trades.push(Trade::new(
                    order.trading_pair.clone(),
                    maker.id.clone(),
                    order.id.clone(),
                    level.price,
                    fill_qty,
                ));
            }

            for i in filled_indices.into_iter().rev() {
                level.orders.remove(i);
            }
        }

        opposite_book.retain(|_, level| !level.is_empty());
    }
}