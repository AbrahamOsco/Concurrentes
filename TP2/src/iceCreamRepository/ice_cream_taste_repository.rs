extern crate actix;
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, Running};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::net::UdpSocket;
use std::process::exit;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use tp2::common::constant::{ADDRESS_ICE_CREAM_TASTE_REPO, TASTES_PATH};
use tp2::common::logic::{
    create_socket_udp, duplicated_socket, join_handler_threads, send_order_secure,
};
use tp2::common::order::{Order, OrderStatus};

pub struct IceCreamTasteRepository {
    repository: HashMap<String, usize>,
    socket: UdpSocket,
    tx_prepare_order: Sender<(Order, String)>,
    rx_prepare_order: Arc<Mutex<Receiver<(Order, String)>>>,
    join_handles: Vec<Option<JoinHandle<()>>>,
}

fn receiver(socket: UdpSocket, address_repository: Addr<IceCreamTasteRepository>) {
    let mut buf = [0; 1024];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, robot_addr)) => {
                let order: Order = match serde_json::from_slice(&buf[..len]) {
                    Ok(order) => order,
                    Err(e) => {
                        eprintln!("Failed to parse order: {:?}", e);
                        exit(1);
                    }
                };
                let robot_id = robot_addr
                    .to_string()
                    .chars()
                    .last()
                    .get_or_insert('.')
                    .to_string();
                println!(
                    "💬 Received a order  ScreenId:{}  OrderId:{}  RobotId:{}.  tastes -> {:?} ",
                    order.screen_id, order.order_id, robot_id, order.tastes
                );
                if let Ok(_content) = address_repository.try_send(TryToProcessOrder {
                    order: order.clone(),
                    robot_addr: robot_addr.to_string(),
                }) {}
            }
            Err(e) => eprintln!("Failed to receive from socket: {:?}", e),
        }
    }
}

fn sender(
    socket_sender: UdpSocket,
    rx_prepare_order_sender: Arc<Mutex<Receiver<(Order, String)>>>,
) {
    loop {
        let (order, robot_address) = match rx_prepare_order_sender.lock() {
            Ok(mutex_rx_prepare_order) => match mutex_rx_prepare_order.recv() {
                Ok((order, robot_address)) => (order, robot_address),
                Err(e) => {
                    eprintln!("Error in recv of rx_prepare_order {}", e);
                    exit(1);
                }
            },
            Err(e) => {
                eprintln!("Error in try to lock rx_prepare_order {}", e);
                exit(1)
            }
        };
        let robot_id = robot_address
            .to_string()
            .chars()
            .last()
            .get_or_insert('.')
            .to_string();
        send_order_secure(
            &socket_sender,
            format!("Repository To RobotId:{}", robot_id).as_str(),
            &order,
            robot_address,
        );
    }
}

impl IceCreamTasteRepository {
    pub fn new() -> Self {
        let mut file = match File::open(TASTES_PATH) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Error to open the file {}", e);
                exit(1);
            }
        };
        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error to read the file {}", e);
                exit(1)
            }
        }

        let repository: HashMap<String, usize> = match serde_json::from_str(&contents) {
            Ok(repo) => repo,
            Err(e) => {
                eprintln!("Error parsing JSON: {}", e);
                exit(1);
            }
        };

        let socket = create_socket_udp(ADDRESS_ICE_CREAM_TASTE_REPO.to_string());
        let (tx_prepare_order, rx_prepare_order) = mpsc::channel::<(Order, String)>();
        IceCreamTasteRepository {
            repository,
            socket,
            tx_prepare_order,
            rx_prepare_order: Arc::new(Mutex::new(rx_prepare_order)),
            join_handles: Vec::new(),
        }
    }
    pub fn start_threads(&mut self, ctx: &mut Context<IceCreamTasteRepository>) {
        println!("🏪❄️Repository starts with {:?} \n", self.repository);
        let socket_receiver = duplicated_socket(&self.socket);
        let socket_sender = duplicated_socket(&self.socket);
        let address_receiver = ctx.address().clone();
        let rx_prepare_order_sender = self.rx_prepare_order.clone();

        // thread receiver
        let join_handler_receiver = thread::spawn(move || {
            receiver(socket_receiver, address_receiver);
        });

        // thread sender
        let join_handler_sender = thread::spawn(move || {
            sender(socket_sender, rx_prepare_order_sender);
        });
        self.join_handles.push(Some(join_handler_receiver));
        self.join_handles.push(Some(join_handler_sender));
    }
}

impl Actor for IceCreamTasteRepository {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_threads(ctx);
    }
    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        join_handler_threads(&mut self.join_handles);
        Running::Stop
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct TryToProcessOrder {
    order: Order,
    robot_addr: String,
}
impl Handler<TryToProcessOrder> for IceCreamTasteRepository {
    type Result = ();
    fn handle(&mut self, msg: TryToProcessOrder, _ctx: &mut Context<Self>) -> Self::Result {
        let mut order = msg.order.clone();
        order.status = OrderStatus::Failed;
        for (taste, necessary_amount) in order.tastes.iter() {
            if self.repository.contains_key(taste) {
                let current_amount = match self.repository.get(taste) {
                    Some(content) => *content,
                    None => 0,
                };
                if current_amount < *necessary_amount {
                    println!(
                        "✖️ Not enough taste: {} idScreen: {} orderId: {} ",
                        taste, order.screen_id, order.order_id
                    );
                    order.message = format!(
                        "✖️ Not enough taste the order need {} {} and the repository has {} {} ",
                        necessary_amount, taste, current_amount, taste
                    );
                    if let Ok(_content) = self.tx_prepare_order.send((order, msg.robot_addr)) {}
                    println!(" 🗃️ Current Stock: {:?}", self.repository);
                    return;
                }
            } else {
                println!("✖️ Taste doesn't exist! Error in the file.");
                order.message = format!("Taste does not exist! File error");
                if let Ok(_content) = self.tx_prepare_order.send((order, msg.robot_addr)) {}
                println!("🗃️ Current Stock: {:?}", self.repository);
                return;
            }
        }
        // si llega aca es porque tenemos todos los tastes then we udpate the stocks.
        for (taste, amount) in order.tastes.iter() {
            let current_amount = match self.repository.get(taste) {
                Some(content) => *content,
                None => 0,
            };
            self.repository
                .insert((*taste).to_string(), current_amount - amount);
        }
        order.status = OrderStatus::OrderReadyToCharge;
        order.message = format!("Ice cream order successfully shipped");
        if let Ok(_content) = self.tx_prepare_order.send((order, msg.robot_addr)) {}
        println!("☑️ Order shipped correctly ");
        println!(" 🗃️ Current Stock: {:?}", self.repository);
    }
}
