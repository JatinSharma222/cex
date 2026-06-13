use std::fmt;

#[derive(Debug)]
pub enum EngineError {

    RedisConnectionFailed(String),
    RedisPopFailed(String),
    RedisPushFailed(String),


    InvalidRequest(String),
    MissingField(String),
    InvalidSymbol(String),
    InvalidSide(String),
    InvalidOrderType(String),


    OrderNotFound(String),
    InsufficientLiquidity,


    SerializationFailed(String),
    DeserializationFailed(String),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::RedisConnectionFailed(msg)  => write!(f, "Redis connection failed: {}", msg),
            EngineError::RedisPopFailed(msg)         => write!(f, "Redis pop failed: {}", msg),
            EngineError::RedisPushFailed(msg)        => write!(f, "Redis push failed: {}", msg),
            EngineError::InvalidRequest(msg)         => write!(f, "Invalid request: {}", msg),
            EngineError::MissingField(field)         => write!(f, "Missing field: {}", field),
            EngineError::InvalidSymbol(symbol)       => write!(f, "Invalid symbol: {}", symbol),
            EngineError::InvalidSide(side)           => write!(f, "Invalid side: {}", side),
            EngineError::InvalidOrderType(ot)        => write!(f, "Invalid order type: {}", ot),
            EngineError::OrderNotFound(id)           => write!(f, "Order not found: {}", id),
            EngineError::InsufficientLiquidity       => write!(f, "Insufficient liquidity"),
            EngineError::SerializationFailed(msg)    => write!(f, "Serialization failed: {}", msg),
            EngineError::DeserializationFailed(msg)  => write!(f, "Deserialization failed: {}", msg),
        }
    }
}


impl std::error::Error for EngineError {}


impl From<redis::RedisError> for EngineError {
    fn from(e: redis::RedisError) -> Self {
        EngineError::RedisConnectionFailed(e.to_string())
    }
}


impl From<serde_json::Error> for EngineError {
    fn from(e: serde_json::Error) -> Self {
        EngineError::DeserializationFailed(e.to_string())
    }
}