mod order_manager;
use crate::order_manager::OrderManager;
use actix::Actor;
use actix_rt::System;

// cargo run --bin order
fn main() {
    let system = System::new();
    system.block_on(async {
        OrderManager::new().start();
    });
    if let Ok(_content) = system.run() {}
}
