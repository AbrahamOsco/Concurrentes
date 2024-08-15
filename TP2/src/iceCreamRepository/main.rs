use actix::Actor;
use actix_rt::System;
mod ice_cream_taste_repository;
use crate::ice_cream_taste_repository::IceCreamTasteRepository;

// cargo run --bin repo
fn main() {
    let system = System::new();
    system.block_on(async {
        IceCreamTasteRepository::new().start();
    });
    if let Ok(_content) = system.run() {}
}
