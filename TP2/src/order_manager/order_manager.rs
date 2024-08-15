use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};
use actix_rt::System;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::process::exit;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use tp2::common::constant::ADDRESS_ORDER_MANAGER;
use tp2::common::logic::StatusRobot::{Busy, Free};
use tp2::common::logic::{
    create_socket_udp, duplicated_socket, get_robot_id, join_handler_threads,
    send_order_by_channel, send_order_secure, ConnectionRobot,
};
use tp2::common::order::{Order, OrderStatus};

pub struct OrderManager {
    socket: UdpSocket,
    connections_intefaces: Arc<Mutex<HashMap<usize, String>>>, // id , address
    connections_robots: Arc<Mutex<Vec<ConnectionRobot>>>,
    tx_start_order: Arc<Mutex<Sender<Order>>>,
    rx_start_order: Arc<Mutex<Receiver<Order>>>,
    tx_end_order: Arc<Mutex<Sender<Order>>>,
    rx_end_order: Arc<Mutex<Receiver<Order>>>,
    tx_decide_robot: Sender<usize>,
    rx_decide_robot: Arc<Mutex<Receiver<usize>>>,
    join_handles: Vec<Option<JoinHandle<()>>>,
}

fn print_connections(
    connects_robots: &Arc<Mutex<Vec<ConnectionRobot>>>,
    connects_interface: &Arc<Mutex<HashMap<usize, String>>>,
) {
    if let Ok(connections_robot) = connects_robots.lock() {
        if let Ok(connections_interface) = connects_interface.lock() {
            print!("Robots:");
            for robot in connections_robot.clone() {
                print!("| 🤖: id:{}  status:{:?} |", robot.id, robot.status)
            }
            print!("\nInterfaces: ");
            for interface in connections_interface.clone() {
                print!("|🖥️ id:{}  |", interface.0)
            }
            print!("\n");
        }
    }
}
fn receiver(
    socket: UdpSocket,
    tx_start: Arc<Mutex<Sender<Order>>>,
    tx_end: Arc<Mutex<Sender<Order>>>,
    connects_interface: Arc<Mutex<HashMap<usize, String>>>,
    connects_robots: Arc<Mutex<Vec<ConnectionRobot>>>,
) {
    println!("OrderManager started! 🕶️🧿");
    let mut buffer = [0; 1024];
    loop {
        print_connections(&connects_robots, &connects_interface);
        match socket.recv_from(&mut buffer) {
            Ok((bytes_size, addr_peer)) => {
                if let Ok(order) = serde_json::from_slice::<Order>(&buffer[..bytes_size]) {
                    if order.status == OrderStatus::InProgress {
                        println!(
                            "Recv a order:📜 From: interface:{}  order_id:{}",
                            order.screen_id, order.order_id
                        );
                        if let Ok(mut mutex_hash_map) = connects_interface.lock() {
                            if !mutex_hash_map.contains_key(&order.screen_id) {
                                mutex_hash_map.insert(order.screen_id, addr_peer.to_string());
                            }
                        }
                        send_order_by_channel(&tx_start, &order, "tx_start");
                    } else if order.status == OrderStatus::Failed
                        || order.status == OrderStatus::OrderReadyToCharge
                    {
                        println!(
                            "Recv a order 🦿 From: robot:{}  order_id:{}",
                            get_robot_id(addr_peer.to_string()),
                            order.order_id
                        );
                        if let Ok(mut connects_robots) = connects_robots.lock() {
                            for robot in connects_robots.iter_mut() {
                                if robot.id == get_robot_id(addr_peer.to_string()) {
                                    robot.status = Free;
                                }
                            }
                        }
                        send_order_by_channel(&tx_end, &order, "tx_end");
                    }
                } else {
                    receiver_string_message_handler(
                        &buffer,
                        &bytes_size,
                        &addr_peer,
                        &connects_robots,
                    );
                }
            }
            Err(e) => {
                eprintln!("Failed to receive message: {}", e);
            }
        }
    }
}

fn receiver_string_message_handler(
    buffer: &[u8; 1024],
    bytes_size: &usize,
    addr_peer: &SocketAddr,
    connects_robots: &Arc<Mutex<Vec<ConnectionRobot>>>,
) {
    let message = match std::str::from_utf8(&buffer[..*bytes_size]) {
        Ok(message) => message,
        Err(e) => {
            eprintln!("Error in convert bytes to string {}", e);
            exit(1)
        }
    };
    if message == "Hello Robot" {
        let robot_id = get_robot_id(addr_peer.to_string());
        let connection_robot = ConnectionRobot {
            id: robot_id,
            address: addr_peer.to_string(),
            status: Free,
        };
        println!("It's a new Robot 🤖 {}! ", robot_id);
        if let Ok(mut mutex_connects_robots) = connects_robots.lock() {
            mutex_connects_robots.push(connection_robot);
        }
    }
}

fn get_index_and_address_robot(
    connections_robots: &Arc<Mutex<Vec<ConnectionRobot>>>,
    decide_robot: &Arc<Mutex<Receiver<usize>>>,
) -> (usize, String, usize) {
    let robot_index_to_work = match decide_robot.lock() {
        Ok(mutex_decide_robot) => match (*mutex_decide_robot).recv() {
            Ok(robot_index) => robot_index,
            Err(e) => {
                eprintln!("Error in try recv decide_robot {}", e);
                exit(1)
            }
        },
        Err(e) => {
            eprintln!("Error in try lock decide_robot {}", e);
            exit(1)
        }
    };

    let (address_robot, robot_id) = match connections_robots.lock() {
        Ok(mutex_connects_robots) => (
            (*mutex_connects_robots)[robot_index_to_work]
                .address
                .to_string(),
            (*mutex_connects_robots)[robot_index_to_work].id,
        ),
        Err(e) => {
            eprintln!("Error in try lock connections_robots {}", e);
            exit(1)
        }
    };
    (robot_index_to_work, address_robot, robot_id)
}

fn sender_to_robots(
    rx_start_order: Arc<Mutex<Receiver<Order>>>,
    socket: UdpSocket,
    connections_robots: Arc<Mutex<Vec<ConnectionRobot>>>,
    decide_robot: Arc<Mutex<Receiver<usize>>>,
    address_order: Addr<OrderManager>,
) {
    loop {
        let order_to_send = match rx_start_order.lock() {
            Ok(mutex_rx_start) => match mutex_rx_start.recv() {
                Ok(order) => order,
                Err(e) => {
                    eprintln!("Error to try recv in mutex_rx_start {:?}", e);
                    exit(1);
                }
            },
            Err(e) => {
                eprintln!("Error to try lock in rx_start {:?}", e);
                exit(1);
            }
        };
        let copy_connect_robots = match connections_robots.lock() {
            Ok(content) => (*content).clone(),
            Err(e) => {
                eprintln!("Error to copy conections robot {}", e);
                exit(1)
            }
        };
        if let Ok(_) = address_order.try_send(PushRobotIndexToPrepareNewOrder {
            current_connections_robots: copy_connect_robots,
        }) {}
        let (robot_index_to_work, address_robot, robot_id) =
            get_index_and_address_robot(&connections_robots, &decide_robot);
        send_order_secure(
            &socket,
            format!("OrderManager Sent Robot id:{}", robot_id).as_str(),
            &order_to_send,
            address_robot.to_string(),
        );
        if let Ok(mut mutex_connect_robots) = connections_robots.lock() {
            mutex_connect_robots[robot_index_to_work].status = Busy;
            println!(
                "Now: 🤖: id:{}  Status:{:?} ",
                get_robot_id(address_robot.to_string()),
                mutex_connect_robots[robot_index_to_work].status
            )
        }
    }
}

impl OrderManager {
    pub fn new() -> OrderManager {
        let (tx_start_order, rx_start_order) = mpsc::channel::<Order>();
        let (tx_end_order, rx_end_order) = mpsc::channel::<Order>();
        let (tx_decide_robot, rx_decide_robot) = mpsc::channel::<usize>();
        let socket = create_socket_udp(ADDRESS_ORDER_MANAGER.to_string());
        OrderManager {
            socket,
            connections_intefaces: Arc::new(Mutex::new(HashMap::new())),
            connections_robots: Arc::new(Mutex::new(vec![])),
            tx_start_order: Arc::new(Mutex::new(tx_start_order)),
            rx_start_order: Arc::new(Mutex::new(rx_start_order)),
            tx_end_order: Arc::new(Mutex::new(tx_end_order)),
            rx_end_order: Arc::new(Mutex::new(rx_end_order)),
            tx_decide_robot,
            rx_decide_robot: Arc::new(Mutex::new(rx_decide_robot)),
            join_handles: vec![],
        }
    }
    fn start_threads(&mut self, ctx: &mut Context<OrderManager>) {
        let socket_receiver = duplicated_socket(&self.socket);
        let socket_sender_robots = duplicated_socket(&self.socket);
        let socket_sender_interfaces = duplicated_socket(&self.socket);
        let tx_start_clone = self.tx_start_order.clone();
        let rx_start_clone = self.rx_start_order.clone();
        let tx_end_clone = self.tx_end_order.clone();
        let rx_end_clone = self.rx_end_order.clone();
        let connects_inter_receiver = self.connections_intefaces.clone();
        let connects_inter_sender = self.connections_intefaces.clone();
        let connections_robots_sender = self.connections_robots.clone();
        let connections_robots_receiver = self.connections_robots.clone();
        let decide_robot_clone_sender = self.rx_decide_robot.clone();
        let address_order = ctx.address();

        let join_handler_recv = thread::spawn(move || {
            receiver(
                socket_receiver,
                tx_start_clone,
                tx_end_clone,
                connects_inter_receiver,
                connections_robots_receiver,
            )
        });
        // Thread sender 1 para el channel rx_start_order enviar las ordenes a los robots y se queda bloqueado en el recv de este channel:
        let join_handler_sender_robot = thread::spawn(move || {
            sender_to_robots(
                rx_start_clone,
                socket_sender_robots,
                connections_robots_sender,
                decide_robot_clone_sender,
                address_order,
            );
        });
        // thread sender 2 para el channel rx_end_order envia las ordenes pa cobrar/failed a las terminales.
        let join_handler_sender_interfaces = thread::spawn(move || {
            sender_to_interfaces(
                socket_sender_interfaces,
                rx_end_clone,
                connects_inter_sender,
            );
        });
        self.join_handles.push(Some(join_handler_recv));
        self.join_handles.push(Some(join_handler_sender_robot));
        self.join_handles.push(Some(join_handler_sender_interfaces));
    }
}

fn sender_to_interfaces(
    socket: UdpSocket,
    rx_end_order: Arc<Mutex<Receiver<Order>>>,
    connects_interfaces: Arc<Mutex<HashMap<usize, String>>>,
) {
    loop {
        let order_to_send = match rx_end_order.lock() {
            Ok(mutex_rx_start) => match mutex_rx_start.recv() {
                Ok(order) => order,
                Err(e) => {
                    eprintln!("Error to try recv in mutex_rx_end {:?}", e);
                    exit(1);
                }
            },
            Err(e) => {
                eprintln!("Erro to try lock in rx_end {:?}", e);
                exit(1);
            }
        };
        let screen_id = order_to_send.screen_id;
        let address_interface = match connects_interfaces.lock() {
            Ok(mtx_connections_interf) => match mtx_connections_interf.get(&screen_id) {
                None => {
                    eprintln!("Error to get the screen id doesn't  exists! ");
                    exit(1);
                }
                Some(addres_interface) => addres_interface.to_string(),
            },
            Err(e) => {
                eprintln!("Error to try lock connections interfaces {}", e);
                exit(1);
            }
        };
        send_order_secure(
            &socket,
            format!(
                "OrderManager Sent  interface {} addres: {}",
                screen_id, address_interface
            )
            .as_str(),
            &order_to_send,
            address_interface,
        );
    }
}

// messages for OrderManagers
#[derive(Message)]
#[rtype(result = "()")]
pub struct PushRobotIndexToPrepareNewOrder {
    current_connections_robots: Vec<ConnectionRobot>,
}

impl Actor for OrderManager {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_threads(ctx);
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        join_handler_threads(&mut self.join_handles);
        System::current().stop();
    }
}

impl Handler<PushRobotIndexToPrepareNewOrder> for OrderManager {
    type Result = ();
    fn handle(
        &mut self,
        msg: PushRobotIndexToPrepareNewOrder,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let robots = &msg.current_connections_robots;
        if robots.len() == 0 {
            println!("ERROR doesn't exist a robot to work! ")
        }
        let mut robot_index = 0;
        let mut there_are_free_robot = false;
        for robot in robots {
            if robot.status == Free {
                there_are_free_robot = true;
                break;
            }
            robot_index += 1;
        }
        if !there_are_free_robot {
            let random_index = thread_rng().gen_range(0..robots.len());
            robot_index = random_index;
        }
        if let Ok(_) = self.tx_decide_robot.send(robot_index) {}
    }
}
