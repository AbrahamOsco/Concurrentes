use crate::common::constant::GRAM_PRICE_OF_ICE_CREAM;
use crate::common::order::{Order, OrderStatus};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::net::UdpSocket;
use std::process::exit;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use super::constant::{ADDRESS_GATEWAY, ADDRESS_INTERFACE, ADDRESS_ROBOT};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum StatusRobot {
    Free,
    Busy,
    Fallen,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ConnectionRobot {
    pub id: usize,
    pub address: String,
    pub status: StatusRobot,
}
pub fn duplicated_socket(socket: &UdpSocket) -> UdpSocket {
    match socket.try_clone() {
        Ok(socket_clone) => socket_clone,
        Err(e) => {
            eprintln!("Error {}", e);
            exit(1);
        }
    }
}

pub fn create_socket_udp(addres: String) -> UdpSocket {
    match UdpSocket::bind(addres) {
        Ok(socket) => socket,
        Err(e) => {
            eprintln!("Failed to bind UDP socket: {}", e);
            exit(1);
        }
    }
}

pub fn send_order_secure(socket: &UdpSocket, from: &str, order: &Order, adress: String) {
    if let Ok(order_string) = serde_json::to_string(&order) {
        if let Ok(_) = socket.send_to(order_string.as_bytes(), adress) {
            println!(
                "📨 {} An order -> ScreenId:{}  OrderId:{}  Status:{:?}, Masa: {} Message:'{}'",
                from,
                order.screen_id,
                order.order_id,
                order.status,
                order.total_mass,
                order.message
            );
        }
    }
}
pub fn send_order_by_channel(
    tx_channel_order: &Arc<Mutex<Sender<Order>>>,
    order: &Order,
    name_channel: &str,
) {
    match tx_channel_order.lock() {
        Ok(mutex_tx_channel_order) => match mutex_tx_channel_order.send(order.clone()) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error to try send in {} {}", name_channel, e);
                exit(1);
            }
        },
        Err(e) => {
            eprintln!("Error to try lock in {} {}", name_channel, e);
            exit(1)
        }
    }
}

pub fn get_robot_id(robot_address: String) -> usize {
    match robot_address.to_string().chars().last() {
        None => {
            eprintln!("Error to get id robot ");
            exit(1);
        }
        Some(id_char) => match id_char.to_digit(10) {
            None => {
                eprintln!("Error to parse id robot ");
                exit(1);
            }
            Some(robot_id) => robot_id as usize,
        },
    }
}

pub fn id_to_addr_interface(id: usize) -> String {
    ADDRESS_INTERFACE.to_owned() + &*id.to_string()
}
pub fn id_to_addr_gateway(id: usize) -> String {
    ADDRESS_GATEWAY.to_owned() + &*id.to_string()
}
pub fn id_to_addr_robot(id: usize) -> String {
    ADDRESS_ROBOT.to_owned() + &*id.to_string()
}

pub fn join_handler_threads(vec_join_handles: &mut Vec<Option<JoinHandle<()>>>) {
    for join_handle in vec_join_handles.iter_mut() {
        if let Some(handle) = join_handle.take() {
            if let Ok(_) = handle.join() {
                println!("A join success!");
            }
        }
    }
}

pub fn parse_orders(data: &Value, orders: &mut Vec<Order>) -> Result<(), Box<dyn Error>> {
    let screen_id = data["pantallaId"]
        .as_u64()
        .ok_or("Expect pantallaId to be a u64")? as usize;
    let orders_json = data["pedidosTotales"]
        .as_array()
        .ok_or("Expected pedidosTotales to be an array")?;
    for a_order in orders_json {
        let order_id = a_order["pedidoId"]
            .as_u64()
            .ok_or("Expect pedidoId to be a u64")? as usize;
        let card_amount = a_order["tarjetaMonto"]
            .as_u64()
            .ok_or("Expect tarjetaMonto to be a u64")? as usize;
        let total_mass = a_order["masaTotal"]
            .as_u64()
            .ok_or("Expect masa_total to be a u64")? as usize;
        let tastes_json = &a_order["gustos"][0];
        let mut tastes = HashMap::new();
        for (key, value) in tastes_json
            .as_object()
            .ok_or("Expected gustos to be an object")?
        {
            if let Some(value) = value.as_u64() {
                tastes.insert(key.clone(), value as usize);
            }
        }
        let order = Order {
            screen_id,
            order_id,
            card_amount,
            total_mass,
            tastes,
            status: OrderStatus::PendingOrder,
            message: "".to_string(),
        };
        orders.push(order);
    }
    Ok(())
}

pub fn get_id_args(name_actor: &str) -> usize {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: cargo run --bin {} <order_number>", name_actor);
        exit(1);
    }
    match args[1].parse() {
        Ok(num) => num,
        Err(e) => {
            eprintln!("Error {}", e);
            exit(1);
        }
    }
}

pub fn get_price_order(total_mass: usize) -> f32 {
    total_mass as f32 * GRAM_PRICE_OF_ICE_CREAM
}
