use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum OrderStatus {
    PendingOrder,
    InProgress,
    Failed,
    OrderReadyToCharge,
    Done,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Order {
    pub screen_id: usize,
    pub order_id: usize,
    pub card_amount: usize,
    pub total_mass: usize,
    pub tastes: HashMap<String, usize>,
    pub status: OrderStatus,
    pub message: String,
}
