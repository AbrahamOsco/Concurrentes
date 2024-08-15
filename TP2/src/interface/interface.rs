extern crate actix;
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, Running};
use actix_rt::System;
use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::net::UdpSocket;
use std::process::exit;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use tp2::common::constant::{ADDRESS_ORDER_MANAGER, NAME_ACTOR_INTERFACE, ORDER_STATUS_CAPTURE, ORDER_STATUS_COMMIT};
use tp2::common::logic::{
    create_socket_udp, duplicated_socket, get_id_args, id_to_addr_gateway, id_to_addr_interface,
    join_handler_threads, parse_orders, send_order_by_channel, send_order_secure,
};
use tp2::common::order::{Order, OrderStatus};

#[derive(Debug)]
pub struct InterfaceUI {
    orders: Vec<Order>,
    index_current_order: usize,
    all_orders_completed: bool,
    id: usize,
    socket: UdpSocket,
    join_handles: Vec<Option<JoinHandle<()>>>,
    receiver_closed: Arc<Mutex<bool>>,
    tx_start_gateway_order: Sender<Order>,
    rx_start_gateway_order: Arc<Mutex<Receiver<Order>>>,
    tx_start_manager_order: Arc<Mutex<Sender<Order>>>,
    rx_start_manager_order: Arc<Mutex<Receiver<Order>>>,
    tx_end_manager_order: Arc<Mutex<Sender<Order>>>,
    rx_end_manager_order: Arc<Mutex<Receiver<Order>>>,
}

fn receiver_is_closed(bool_mutex: &Arc<Mutex<bool>>) -> bool {
    match bool_mutex.lock() {
        Ok(condition) => *condition,
        Err(e) => {
            eprintln!("Error in lock {}", e);
            false
        }
    }
}

fn sender_manager(socket: UdpSocket, rx_start_manager_order: Arc<Mutex<Receiver<Order>>>) {
    loop {
        let order = match rx_start_manager_order.lock() {
            Ok(mtx_order) => match mtx_order.recv() {
                Ok(order) => order,
                Err(e) => {
                    eprintln!("Error try to lock the rx_start_manager_order {}", e);
                    exit(1);
                }
            },
            Err(e) => {
                eprintln!("Error try to lock the rx_start_manager_order {}", e);
                exit(1);
            }
        };
        let message = format!("Interface {} Sends Order Manager 📜", order.screen_id);
        send_order_secure(
            &socket,
            message.as_str(),
            &order,
            ADDRESS_ORDER_MANAGER.to_string(),
        );
    }
}

fn sender_gateway_start(socket: UdpSocket, rx_start_gateway_order: Arc<Mutex<Receiver<Order>>>) {
    loop {
        let order = match rx_start_gateway_order.lock() {
            Ok(mtx_order) => match mtx_order.recv() {
                Ok(order) => order,
                Err(e) => {
                    eprintln!("Error try to lock the rx_start_gateway_order {}", e);
                    exit(1);
                }
            },
            Err(e) => {
                eprintln!("Error try to lock the rx_start_gateway_order {}", e);
                exit(1);
            }
        };
        let message = format!(
            "Interface {} sends gateway 🏦 {} to capture payment!📇",
            order.screen_id,
            id_to_addr_gateway(order.screen_id)
        );
        send_order_secure(
            &socket,
            message.as_str(),
            &order,
            id_to_addr_gateway(order.screen_id),
        );
    }
}

fn sender_gateway_end(socket: UdpSocket, rx_end_manager_order: Arc<Mutex<Receiver<Order>>>) {
    loop {
        let order = match rx_end_manager_order.lock() {
            Ok(mtx_order) => match mtx_order.recv() {
                Ok(order) => order,
                Err(e) => {
                    eprintln!("Error try to lock the rx_end_manager_order {}", e);
                    exit(1);
                }
            },
            Err(e) => {
                eprintln!("Error try to lock the rx_end_manager_order {}", e);
                exit(1);
            }
        };
        let message = format!("Interface {} Sent to gateway 🔐", order.screen_id);
        send_order_secure(
            &socket,
            message.as_str(),
            &order,
            id_to_addr_gateway(order.screen_id),
        );
    }
}

fn receiver(
    bool_close_reciever: Arc<Mutex<bool>>,
    socket: UdpSocket,
    address_interface: Addr<InterfaceUI>,
    tx_end_manager_order: Arc<Mutex<Sender<Order>>>,
    tx_start_manager_order: Arc<Mutex<Sender<Order>>>,
) {
    let mut buffer = [0; 1024];
    while !receiver_is_closed(&bool_close_reciever) {
        match socket.recv_from(&mut buffer) {
            Ok((bytes_size, _addr_peer)) => {
                if let Ok(order) = serde_json::from_slice::<Order>(&buffer[..bytes_size]) {
                    process_received_order(
                        &order,
                        &tx_end_manager_order,
                        &socket,
                        &tx_start_manager_order,
                        &address_interface,
                    );
                }
            }
            Err(e) => {
                eprintln!("Failed to receive message: {}", e)
            }
        }
    }
}

fn process_received_order(
    order: &Order,
    tx_end_manager_order: &Arc<Mutex<Sender<Order>>>,
    socket: &UdpSocket,
    tx_start_manager_order: &Arc<Mutex<Sender<Order>>>,
    address_interface: &Addr<InterfaceUI>,
) {
    if order.status == OrderStatus::InProgress {
        send_order_by_channel(tx_start_manager_order, order, "tx_start_manager_order");
        return;
    }
    if order.status == OrderStatus::Failed {
        println!(
            "Recv an order Failed! ✖️🔴 ScreenId:{} OrderId:{} Status:{:?}. Reason: '{}' \n",
            order.screen_id, order.order_id, order.status, order.message
        );
        if let Ok(_content) = address_interface.try_send(SendNextOrder {}) {}
        return;
    }

    // Orden OrderReadyToCharge next order.
    let mut order = order.clone();
    order.message = ORDER_STATUS_COMMIT.to_string(); // lo enviamos listo para commitear.
    send_order_by_channel(tx_end_manager_order, &order, "tx_end_manager_order");

    let mut buffer = [0; 1024];
    println!(
        "Recv an order 🍨🔐 We need to cancel the payment at the gateway! ScreenId:{} OrderId:{}\
     Status:{:?}. Message: '{}'",
        order.screen_id, order.order_id, order.status, order.message
    );
    match socket.recv_from(&mut buffer) {
        Ok((bytes_size, _addr_gateway)) => {
            if let Ok(order) = serde_json::from_slice::<Order>(&buffer[..bytes_size]) {
                println!("Recv an order Finished!🍨☑️💫 ScreenId:{} OrderId:{} Status:{:?}. Message: '{}' \n", order.screen_id,
                         order.order_id, order.status, order.message);
                if let Ok(_content) = address_interface.try_send(SendNextOrder {}) {}
            }
        }
        Err(e) => {
            eprintln!("Error to try lock in tx_end_manager_order {}", e);
            exit(1)
        }
    }
}

impl InterfaceUI {
    pub fn new() -> InterfaceUI {
        let id = get_id_args(NAME_ACTOR_INTERFACE);
        let socket = create_socket_udp(id_to_addr_interface(id));
        let (tx_start_gateway_order, rx_start_gateway_order) = mpsc::channel::<Order>();
        let (tx_start_manager_order, rx_start_manager_order) = mpsc::channel::<Order>();
        let (tx_end_manager_order, rx_end_manager_order) = mpsc::channel::<Order>();

        InterfaceUI {
            orders: Vec::new(),
            id,
            socket,
            index_current_order: 0,
            all_orders_completed: false,
            join_handles: Vec::new(),
            tx_start_gateway_order,
            rx_start_gateway_order: Arc::new(Mutex::new(rx_start_gateway_order)),
            tx_start_manager_order: Arc::new(Mutex::new(tx_start_manager_order)),
            rx_start_manager_order: Arc::new(Mutex::new(rx_start_manager_order)),
            receiver_closed: Arc::new(Mutex::new(false)),
            tx_end_manager_order: Arc::new(Mutex::new(tx_end_manager_order)),
            rx_end_manager_order: Arc::new(Mutex::new(rx_end_manager_order)),
        }
    }

    pub fn start_threads(&mut self, ctx: &mut Context<InterfaceUI>) {
        let socket_clone_receiver = duplicated_socket(&self.socket);
        let socket_clone_sender_manager = duplicated_socket(&self.socket);
        let socket_clone_sender_gateway_end = duplicated_socket(&self.socket);
        let socket_clone_sender_gateway_start = duplicated_socket(&self.socket);

        let bool_close_reciever = self.receiver_closed.clone();
        let address_interface_receiver = ctx.address();
        let rx_start_gateway_order_clone = self.rx_start_gateway_order.clone();
        let rx_start_manager_order_clone = self.rx_start_manager_order.clone();
        let tx_start_manager_order_clone = self.tx_start_manager_order.clone();
        let tx_end_manager_order_clone = self.tx_end_manager_order.clone();
        let rx_end_manager_order_clone = self.rx_end_manager_order.clone();

        // thread sender gateway start.
        let join_handler_sender_gateway_start = thread::spawn(move || {
            sender_gateway_start(
                socket_clone_sender_gateway_start,
                rx_start_gateway_order_clone,
            );
        });

        // thread receiver.
        let join_handler_receiver = thread::spawn(move || {
            receiver(
                bool_close_reciever,
                socket_clone_receiver,
                address_interface_receiver,
                tx_end_manager_order_clone,
                tx_start_manager_order_clone,
            )
        });
        // thread sender manager.
        let join_handler_sender_manager = thread::spawn(move || {
            sender_manager(socket_clone_sender_manager, rx_start_manager_order_clone)
        });

        // thread sender gateway end.
        let join_handler_sender_gateway_end = thread::spawn(move || {
            sender_gateway_end(socket_clone_sender_gateway_end, rx_end_manager_order_clone);
        });

        if let Ok(_) = ctx.address().try_send(SendNextOrder {}) {}

        self.join_handles
            .push(Some(join_handler_sender_gateway_start));
        self.join_handles.push(Some(join_handler_receiver));
        self.join_handles.push(Some(join_handler_sender_manager));
        self.join_handles
            .push(Some(join_handler_sender_gateway_end));
    }

    fn close_threads(&mut self) {
        if let Ok(mut content) = self.receiver_closed.lock() {
            *content = true;
        }
        join_handler_threads(&mut self.join_handles);
    }

    fn load_orders(&mut self) {
        if let Ok(orders) = self.get_vec_orders() {
            self.orders = orders;
        }
    }

    fn get_vec_orders(&mut self) -> Result<Vec<Order>, Box<dyn Error>> {
        let filename = format!("orders/orders{}.json", self.id);
        let mut file = File::open(filename)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let data: Value = serde_json::from_str(&contents)?;
        let mut orders = Vec::new();
        match parse_orders(&data, &mut orders) {
            Ok(_) => Ok(orders),
            Err(e) => Err(e),
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct SendNextOrder();

impl Actor for InterfaceUI {
    type Context = Context<Self>; //SyncContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.load_orders();
        self.start_threads(ctx);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        join_handler_threads(&mut self.join_handles);
        Running::Stop
    }
}

impl Handler<SendNextOrder> for InterfaceUI {
    type Result = ();
    fn handle(&mut self, _msg: SendNextOrder, _ctx: &mut Self::Context) -> Self::Result {
        if self.index_current_order == self.orders.len() {
            println!("All orders were completed 💯✔️");
            self.all_orders_completed = true;
            self.close_threads();
            System::current().stop();
            return;
        }
        let mut current_order = self.orders[self.index_current_order].clone();
        current_order.status = OrderStatus::InProgress;
        current_order.message = ORDER_STATUS_CAPTURE.to_string();
        match self.tx_start_gateway_order.send(current_order) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error in send the current_order! {}", e);
                exit(1);
            }
        }
        self.index_current_order += 1;
    }
}
