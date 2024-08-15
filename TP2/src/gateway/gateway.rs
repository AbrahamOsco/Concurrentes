use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};
use rand::Rng;
use std::net::UdpSocket;
use std::process::exit;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;
use tp2::common::constant::{NAME_ACTOR_GATEWAY, ORDER_STATUS_ABORT, ORDER_STATUS_COMMIT, ORDER_STATUS_PREPARE};
use tp2::common::logic::{
    create_socket_udp, duplicated_socket, get_id_args, get_price_order, id_to_addr_gateway,
    id_to_addr_interface, send_order_by_channel, send_order_secure,
};
use tp2::common::order::{Order, OrderStatus};

pub struct Gateway {
    id: usize,
    accumulated_money: f32,
    blocked_money: f32,
    socket: UdpSocket,
    tx_start_order: Sender<Order>,
    rx_start_orer: Arc<Mutex<Receiver<Order>>>,
    rx_end_order: Arc<Mutex<Receiver<Order>>>,
    tx_end_order: Arc<Mutex<Sender<Order>>>,
    join_handles: Vec<Option<JoinHandle<()>>>,
}

fn receiver(
    socket: UdpSocket,
    gateway_address: Addr<Gateway>,
    tx_end_order: Arc<Mutex<Sender<Order>>>,
) {
    let mut buffer = [0; 1024];
    loop {
        match socket.recv_from(&mut buffer) {
            Ok((bytes_size, _addr_peer)) => {
                if let Ok(order) = serde_json::from_slice::<Order>(&buffer[..bytes_size]) {
                    proccess_order(&order, &gateway_address, &tx_end_order);
                }
            }
            Err(e) => {
                eprintln!("Failed to receive message: {}", e)
            }
        }
    }
}

fn proccess_order(
    order: &Order,
    gateway_address: &Addr<Gateway>,
    tx_end_order: &Arc<Mutex<Sender<Order>>>,
) {
    if order.status == OrderStatus::InProgress {
        match gateway_address.try_send(CapturePayment {
            order: order.clone(),
        }) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error in try_send message CapturePayment {}", e);
                exit(1)
            }
        }
    } else if order.status == OrderStatus::OrderReadyToCharge {
        let mut order_udpate = order.clone();
        order_udpate.status = OrderStatus::Failed; // suponemos q nos mandaron un ABORT.
        if order_udpate.message == ORDER_STATUS_COMMIT {
            // si fue un commit lo pasamos a done
            order_udpate.status = OrderStatus::Done;
        }
        match gateway_address.try_send(UpdateAccumulatedMoney {
            order: order_udpate.clone(),
        }) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error in try_send message CapturePayment {}", e);
                exit(1)
            }
        }
        send_order_by_channel(tx_end_order, &order_udpate, "tx_end_order");
    }
}

fn sender_start(socket: UdpSocket, rx_start_order: Arc<Mutex<Receiver<Order>>>, gateway_id: usize) {
    loop {
        let order = match rx_start_order.lock() {
            Ok(mtx_order) => match mtx_order.recv() {
                Ok(order) => order,
                Err(e) => {
                    eprintln!("Error try to lock the rx_start_order {}", e);
                    exit(1);
                }
            },
            Err(e) => {
                eprintln!("Error try to lock the rx_start_order {}", e);
                exit(1);
            }
        };
        println!();
        let message = format!(
            "Initially gateway {} Sent To Interface {} 📜",
            gateway_id, order.screen_id
        );
        send_order_secure(
            &socket,
            message.as_str(),
            &order,
            id_to_addr_interface(order.screen_id),
        );
    }
}

fn sender_end(socket: UdpSocket, rx_end_order: Arc<Mutex<Receiver<Order>>>, gateway_id: usize) {
    loop {
        let order = match rx_end_order.lock() {
            Ok(mtx_order) => match mtx_order.recv() {
                Ok(order) => order,
                Err(e) => {
                    eprintln!("Error try to lock the rx_end_order {}", e);
                    exit(1);
                }
            },
            Err(e) => {
                eprintln!("Error try to lock the rx_end_order {}", e);
                exit(1);
            }
        };
        let message = format!(
            "Finally gateway {} Sent To Interface {} 🫳 ",
            gateway_id, order.screen_id
        );
        send_order_secure(
            &socket,
            message.as_str(),
            &order,
            id_to_addr_interface(order.screen_id),
        );
    }
}

impl Gateway {
    pub fn new() -> Gateway {
        let id = get_id_args(NAME_ACTOR_GATEWAY);
        let socket = create_socket_udp(id_to_addr_gateway(id));
        let (tx_start_order, rx_start_order) = mpsc::channel::<Order>();
        let (tx_end_order, rx_end_order) = mpsc::channel::<Order>();
        Gateway {
            id,
            accumulated_money: 0.0,
            blocked_money: 0.0,
            socket,
            tx_start_order,
            tx_end_order: Arc::new(Mutex::new(tx_end_order)), // este puede ser mutex o no depende del comportamiento lo vemos luego.
            rx_start_orer: Arc::new(Mutex::new(rx_start_order)),
            rx_end_order: Arc::new(Mutex::new(rx_end_order)),
            join_handles: vec![],
        }
    }

    fn start_threads(&mut self, ctx: &mut Context<Gateway>) {
        println!("Gateway started 🏦💸");
        let socket_clone_receiver = duplicated_socket(&self.socket);
        let socket_clone_sender_start = duplicated_socket(&self.socket);
        let socket_clone_sender_end = duplicated_socket(&self.socket);
        let rx_start_order = self.rx_start_orer.clone();
        let rx_end_order = self.rx_end_order.clone();
        let tx_end_order = self.tx_end_order.clone();
        let gateway_address = ctx.address();
        let copy_id = self.id;

        let join_handle_receiver = std::thread::spawn(move || {
            receiver(socket_clone_receiver, gateway_address, tx_end_order);
        });

        let join_handler_sender_start = std::thread::spawn(move || {
            sender_start(socket_clone_sender_start, rx_start_order, copy_id.clone());
        });

        let join_handler_sender_end = std::thread::spawn(move || {
            sender_end(socket_clone_sender_end, rx_end_order, copy_id.clone());
        });

        self.join_handles.push(Some(join_handle_receiver));
        self.join_handles.push(Some(join_handler_sender_start));
        self.join_handles.push(Some(join_handler_sender_end));
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct CapturePayment {
    order: Order,
}

#[derive(Message)]
#[rtype(result = "()")]
struct UpdateAccumulatedMoney {
    order: Order,
}

impl Actor for Gateway {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_threads(ctx)
    }
}
impl Handler<CapturePayment> for Gateway {
    type Result = ();

    fn handle(&mut self, msg: CapturePayment, _ctx: &mut Self::Context) -> Self::Result {
        let mut order = msg.order.clone();
        order.status = OrderStatus::Failed;
        if (order.card_amount as f32) < get_price_order(order.total_mass) {
            order.message = format!("Insufficient funds on the card to pay for the order! you have {} and you need: {} ✖️", order.card_amount, get_price_order(order.total_mass));
            if let Ok(_con) = self.tx_start_order.send(order.clone()) {
                return;
            };
        }
        order.message = "Randomly declined card 🤡".to_string();
        let random_number: f64 = rand::thread_rng().gen();
        if random_number > 0.25 {
            order.status = OrderStatus::InProgress;
            order.message = ORDER_STATUS_PREPARE.to_string();
            self.blocked_money += get_price_order(order.total_mass);
        }
        if let Ok(_con) = self.tx_start_order.send(order) {
            return;
        };
    }
}

impl Handler<UpdateAccumulatedMoney> for Gateway {
    type Result = ();

    fn handle(&mut self, msg: UpdateAccumulatedMoney, _ctx: &mut Self::Context) -> Self::Result {
        if msg.order.message == ORDER_STATUS_COMMIT {
            println!(
                "Committed operation we earn ${} for the order!🤑",
                self.blocked_money
            );
            self.accumulated_money += self.blocked_money;
            self.blocked_money = 0.0;
        } else if msg.order.message == ORDER_STATUS_ABORT {
            println!("Aborted operation, ${} is unlocked.🗿", self.blocked_money);
            self.accumulated_money = 0.0;
        }
        println!(
            "The total balance of the gateway is ${} .💸🏦",
            self.accumulated_money
        );
    }
}
