use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, Running};
use std::net::UdpSocket;
use std::process::exit;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use tp2::common::constant::{
    ADDRESS_ICE_CREAM_TASTE_REPO, ADDRESS_ORDER_MANAGER, NAME_ACTOR_ROBOT,
};
use tp2::common::logic::{
    create_socket_udp, duplicated_socket, get_id_args, id_to_addr_robot, join_handler_threads,
    send_order_by_channel, send_order_secure,
};
use tp2::common::order::{Order, OrderStatus};
use tp2::ring_dist_mutex::ring_dist_mutex::RingDistMutex;

const ROBOTS_MAX: usize = 3;
pub struct Robot {
    id: usize,
    socket: UdpSocket,
    join_handles: Vec<Option<JoinHandle<()>>>,
    mutex_dist: Arc<Mutex<RingDistMutex>>,
    tx_start_order: Arc<Mutex<Sender<Order>>>,
    rx_start_order: Arc<Mutex<Receiver<Order>>>,
    tx_end_order: Arc<Mutex<Sender<Order>>>,
    rx_end_order: Arc<Mutex<Receiver<Order>>>,
    tx_wait_order: Sender<u64>,
    rx_wait_order: Arc<Mutex<Receiver<u64>>>,
}

impl Robot {
    pub fn new() -> Robot {
        let id = get_id_args(NAME_ACTOR_ROBOT);
        let socket = create_socket_udp(id_to_addr_robot(id));
        let (tx_start_order, rx_start_order) = mpsc::channel::<Order>();
        let (tx_end_order, rx_end_order) = mpsc::channel::<Order>();
        let (tx_wait_order, rx_wait_order) = mpsc::channel::<u64>();
        Robot {
            id,
            socket,
            join_handles: Vec::new(),
            tx_start_order: Arc::new(Mutex::new(tx_start_order)),
            rx_start_order: Arc::new(Mutex::new(rx_start_order)),
            mutex_dist: Arc::new(Mutex::new(RingDistMutex::new(id, ROBOTS_MAX))),
            tx_end_order: Arc::new(Mutex::new(tx_end_order)),
            rx_end_order: Arc::new(Mutex::new(rx_end_order)),
            tx_wait_order,
            rx_wait_order: Arc::new(Mutex::new(rx_wait_order)),
        }
    }

    fn start_threads(&mut self, ctx: &mut Context<Robot>) {
        let socket_receiver = duplicated_socket(&self.socket);
        let socket_sender = duplicated_socket(&self.socket);
        let mutex_dist_send = self.mutex_dist.clone();
        let mutex_dist_recv = self.mutex_dist.clone();
        let address_robot_recv = ctx.address();
        let address_robot_send = ctx.address();
        let tx_start_order_clone = self.tx_start_order.clone();
        let rx_start_order_clone = self.rx_start_order.clone();
        let tx_end_order_clone = self.tx_end_order.clone();
        let rx_end_order_clone = self.rx_end_order.clone();
        let rx_wait_order_clone = self.rx_wait_order.clone();
        let robot_id = self.id;
        let join_sender = thread::spawn(move || {
            sender_robot(
                socket_sender,
                address_robot_recv,
                mutex_dist_send,
                rx_start_order_clone,
                rx_end_order_clone,
                robot_id,
            )
        });
        let join_receiver = thread::spawn(move || {
            receiver_robot(
                socket_receiver,
                address_robot_send,
                mutex_dist_recv,
                tx_start_order_clone,
                tx_end_order_clone,
                rx_wait_order_clone,
            )
        });
        self.join_handles.push(Some(join_sender));
        self.join_handles.push(Some(join_receiver));
        println!("Se lanzarons los threads en el robot! !");
    }
}
fn sender_robot(
    socket: UdpSocket,
    _addr_robot: Addr<Robot>,
    mutex_dist: Arc<Mutex<RingDistMutex>>,
    rx_start_order: Arc<Mutex<Receiver<Order>>>,
    rx_end_order: Arc<Mutex<Receiver<Order>>>,
    robot_id: usize,
) {
    // enviamos una unica vez el Hello Robot
    socket
        .send_to(b"Hello Robot", ADDRESS_ORDER_MANAGER)
        .expect("Failed to send message");
    match mutex_dist.lock() {
        Ok(mtx_distrib) => mtx_distrib.start(),
        Err(e) => {
            eprintln!("Error to try start RingDistMutex {}", e);
            exit(1);
        }
    }

    println!("Entrando al thread sender!");
    loop {
        let order_to_repo = match rx_start_order.lock() {
            Ok(content) => match content.recv() {
                Ok(order) => order,
                Err(e) => {
                    eprintln!("Erro to try recv in Receiver {}", e);
                    exit(1);
                }
            },
            Err(e) => {
                eprintln!("Error to try lock rx_start_order {}", e);
                exit(1)
            }
        };
        match mutex_dist.lock() {
            Ok(mut mtx_ring_dist) => mtx_ring_dist.acquire(),
            Err(e) => {
                eprintln!("Error to try lock mtx ring dist! {} ", e);
                exit(1);
            }
        }

        let message_repo = format!("From Robot {} send IceCreamRepo ", robot_id);
        send_order_secure(
            &socket,
            message_repo.as_str(),
            &order_to_repo,
            ADDRESS_ICE_CREAM_TASTE_REPO.to_string(),
        );
        match mutex_dist.lock() {
            Ok(mut mtx_ring_dist) => {
                mtx_ring_dist.release();
            }
            Err(e) => {
                eprintln!("Error to try lock mtx ring dist! {} ", e);
                exit(1);
            }
        }

        if let Ok(mtx_rx_end_order) = rx_end_order.lock() {
            if let Ok(order_to_manager) = mtx_rx_end_order.recv() {
                let message_manager = format!("From Robot {} send OrderManager ", robot_id);
                send_order_secure(
                    &socket,
                    message_manager.as_str(),
                    &order_to_manager,
                    ADDRESS_ORDER_MANAGER.to_string(),
                );
            }
        }
    }
}

fn receiver_robot(
    socket: UdpSocket,
    addr_robot: Addr<Robot>,
    _mutex_dist: Arc<Mutex<RingDistMutex>>,
    tx_start_order: Arc<Mutex<Sender<Order>>>,
    tx_end_order: Arc<Mutex<Sender<Order>>>,
    rx_wait_order: Arc<Mutex<Receiver<u64>>>,
) {
    let mut buffer = [0; 1024];
    loop {
        println!("[Robot] Waiting for message...");
        match socket.recv_from(&mut buffer) {
            Ok((bytes_size, _addr_peer)) => {
                if let Ok(order) = serde_json::from_slice::<Order>(&buffer[..bytes_size]) {
                    println!("[Receiver] Recv a order: {:?}", order);
                    if order.status == OrderStatus::InProgress {
                        send_order_by_channel(&tx_start_order, &order, "tx_start_order");
                    } else if order.status == OrderStatus::Failed
                        || order.status == OrderStatus::OrderReadyToCharge
                    {
                        if order.status == OrderStatus::OrderReadyToCharge {
                            let order_clone = order.clone();
                            match addr_robot.try_send(CoolingTheEngine { order: order_clone }) {
                                Ok(_) => {}
                                Err(e) => {
                                    eprint!(" Error in try_send: CoolingTheEngine {} ", e);
                                    exit(1);
                                }
                            }
                            let time_to_sleep_sec = match rx_wait_order.lock() {
                                Ok(mtx_rx_wait) => match mtx_rx_wait.recv() {
                                    Ok(time) => time,
                                    Err(e) => {
                                        eprintln!("Error to try recv in rx_wait {}", e);
                                        exit(1);
                                    }
                                },
                                Err(e) => {
                                    eprintln!("Error to try lock rx_wait_order {}", e);
                                    exit(1);
                                }
                            };
                            println!("Time to sleep {}", time_to_sleep_sec);
                            thread::sleep(Duration::from_secs(time_to_sleep_sec));
                            println!("I wake UP {}", time_to_sleep_sec);
                        }
                        send_order_by_channel(&tx_end_order, &order, "tx_end_order");
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to receive message: {}", e);
            }
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct CoolingTheEngine {
    order: Order,
}

impl Actor for Robot {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_threads(ctx);
    }
    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        join_handler_threads(&mut self.join_handles);
        Running::Stop
    }
}

impl Handler<CoolingTheEngine> for Robot {
    type Result = ();

    fn handle(&mut self, msg: CoolingTheEngine, _ctx: &mut Self::Context) -> Self::Result {
        let time_to_sleep_in_sec = (msg.order.total_mass as f32 / 1000.0f32) as u64;
        if let Ok(_content) = self.tx_wait_order.send(time_to_sleep_in_sec) {}
    }
}
