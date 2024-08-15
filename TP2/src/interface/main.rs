mod interface;
use crate::interface::InterfaceUI;
use actix::Actor;
use actix_rt::System;
extern crate actix;

// cargo run --bin interface 0 idem con 1,2,3,4,5
fn main() {
    let system = System::new();
    system.block_on(async {
        InterfaceUI::new().start();
    });
    if let Ok(_content) = system.run() {}
}
