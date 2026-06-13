use redis::{Client, Commands, Connection};
use crate::models::{EngineRequest, EngineResponse};

pub struct RedisClient {
    pub connection: Connection,
}

impl RedisClient {

    pub fn publish(&mut self, channel: &str, data: serde_json::Value) {
    let payload = serde_json::to_string(&data)
        .expect("Failed to serialize publish payload");

    let _: () = self.connection
        .publish(channel, payload)
        .expect("Failed to publish to channel");
}

    pub fn new(redis_url: &str) -> Self {
        let client = Client::open(redis_url).expect("Failed to connect to Redis");
        let connection = client
            .get_connection()
            .expect("Failed to get Redis connection");
        Self { connection }
    }

    pub fn pop_request(&mut self, queue: &str) -> Option<EngineRequest> {
        let result: Option<(String, String)> = self.connection.brpop(queue, 0.0).ok();
        match result {
            Some((_, payload)) => {
                match serde_json::from_str::<EngineRequest>(&payload) {
                    Ok(request) => Some(request),
                    Err(e) => {
                        eprintln!("Failed to parse engine request: {}", e);
                        None
                    }
                }
            }
            None => None,
        }
    }

    pub fn push_response(&mut self, queue: &str, response: &EngineResponse) {
        let payload = serde_json::to_string(response)
            .expect("Failed to serialize engine response");
        let _: () = self.connection.lpush(queue, payload)
            .expect("Failed to push response");
    }

    pub fn send_success(&mut self, correlation_id: &str, response_queue: &str, data: serde_json::Value) {
        self.push_response(response_queue, &EngineResponse {
            correlation_id: correlation_id.to_string(),
            success: true,
            data: Some(data),
            error: None,
        });
    }

    pub fn send_error(&mut self, correlation_id: &str, response_queue: &str, error: &str) {
        self.push_response(response_queue, &EngineResponse {
            correlation_id: correlation_id.to_string(),
            success: false,
            data: None,
            error: Some(error.to_string()),
        });
    }
}                                         