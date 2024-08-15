use actix::Actor;
use actix_rt::System;

mod robot;
use crate::robot::Robot;

// cargo run --bin robot 2  /en orden descendiente invocar a los robots desde 3 hasta 0.
fn main() {
    let system = System::new();
    system.block_on(async {
        Robot::new().start();
    });
    if let Ok(_content) = system.run() {}
}
