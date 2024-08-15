use crate::gateway::Gateway;
use actix::Actor;
use actix_rt::System;
mod gateway;

// cargo run --bin gateway 0
fn main() {
    let system = System::new();
    system.block_on(async {
        Gateway::new().start();
    });
    if let Ok(_content) = system.run() {}
}
