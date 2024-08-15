use chrono::Local;
use std::net::UdpSocket;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(3);

fn id_to_addr_ring(id: usize) -> String {
    "127.0.0.1:1345".to_owned() + &*id.to_string()
}

fn current_time() -> String {
    let now = Local::now();
    now.format("%H:%M:%S").to_string()
}
pub struct RingDistMutex {
    id: usize,
    ring_size: usize,
    socket: UdpSocket,
    lock_needed: Arc<(Mutex<bool>, Condvar)>,
    has_token: Arc<(Mutex<bool>, Condvar)>,
    got_ack: Arc<(Mutex<bool>, Condvar)>,
    handles: Vec<thread::JoinHandle<()>>,
}
impl RingDistMutex {
    pub fn new(id: usize, ring_size: usize) -> RingDistMutex {
        RingDistMutex {
            id,
            ring_size,
            socket: UdpSocket::bind(id_to_addr_ring(id)).unwrap(),
            lock_needed: Arc::new((Mutex::new(false), Condvar::new())),
            has_token: Arc::new((Mutex::new(false), Condvar::new())),
            got_ack: Arc::new((Mutex::new(false), Condvar::new())),
            handles: Vec::new(),
        }
    }
    pub fn start(&self) {
        let mut ret_clone = self.clone();
        thread::spawn(move || ret_clone.receiver()); // falta pushear y joinear aca.
    }

    pub fn clone(&self) -> RingDistMutex {
        RingDistMutex {
            id: self.id,
            ring_size: self.ring_size,
            socket: self.socket.try_clone().unwrap(),
            lock_needed: self.lock_needed.clone(),
            has_token: self.has_token.clone(),
            got_ack: self.got_ack.clone(),
            handles: Vec::new(),
        }
    }
    pub fn acquire(&mut self) {
        *self.lock_needed.0.lock().unwrap() = true;
        self.lock_needed.1.notify_all();
        self.has_token
            .1
            .wait_while(self.has_token.0.lock().unwrap(), |has_it| !*has_it);
        *self.got_ack.0.lock().unwrap() = false;
        self.got_ack.1.notify_all();
        println!("[Acquire🤏] id:{}!] [{}]", self.id, current_time());
    }

    pub fn release(&mut self) {
        *self.lock_needed.0.lock().unwrap() = false;
        self.lock_needed.1.notify_all();
        println!("[Release🫴🏻] id:{} [{}]", self.id, current_time());
    }
    fn next_id(&self, id: usize) -> usize {
        (id + 1) % self.ring_size
    }

    fn send_safe_token(&mut self, id: usize) {
        let msg = "TOKEN";
        let next_id = self.next_id(id);
        if next_id == self.id {
            // si el sgt soy yo el anillo esta vacio ->estoy solo.
            println!("[RECEIVER {}] Send msg{} a {}", self.id, msg, next_id);
            eprintln!("I went all the way around without answers");
            std::process::exit(1);
        }
        println!(
            "[RECEIVER 📨{}] Send Token To id:{} [{}]",
            self.id,
            next_id,
            current_time()
        );
        self.socket
            .send_to(msg.as_bytes(), id_to_addr_ring(next_id))
            .unwrap();
        let got_ack = self.got_ack.1.wait_timeout_while(
            self.got_ack.0.lock().unwrap(),
            TIMEOUT,
            |got_ack_bool| !*got_ack_bool,
        );
        if got_ack.unwrap().1.timed_out() {
            println!("TIMEOUT:!💤  Robot{} [{}]", next_id, current_time());
            self.send_safe_token(next_id);
        }
    }

    fn receiver(&mut self) {
        if self.id == 0 {
            self.socket
                .send_to("TOKEN".as_bytes(), id_to_addr_ring(self.id))
                .unwrap();
        }
        loop {
            let mut buffer = [0; 10];
            let (size, from) = self.socket.recv_from(&mut buffer).unwrap();
            let message = std::str::from_utf8(&buffer[..size])
                .expect("Error al convertir los bytes a String");
            if message == "TOKEN" {
                println!(
                    "[RECEIVER {} 📩] Recv: TOKEN from:{}  [{}]",
                    self.id,
                    from,
                    current_time()
                );
                if let Some(last_char) = from.to_string().chars().last() {
                    let last_digit = last_char.to_digit(10).unwrap_or(10);
                    if last_digit != self.id as u32 {
                        self.socket.send_to("ACK".as_bytes(), from).unwrap();
                        println!(
                            "[RECEIVER 🤚{}] Send ACK TO :{} [{}] ",
                            self.id,
                            from,
                            current_time()
                        );
                    }
                }
                *self.has_token.0.lock().unwrap() = true;
                self.has_token.1.notify_all();
                self.lock_needed
                    .1
                    .wait_while(self.lock_needed.0.lock().unwrap(), |needs_it| *needs_it);
                *self.has_token.0.lock().unwrap() = false;
                self.has_token.1.notify_all();
                thread::sleep(Duration::from_millis(1500));
                let mut clone = self.clone(); // clono a mi mismo.
                let handle_join = thread::spawn(move || clone.send_safe_token(clone.id));
                self.handles.push(handle_join);
            } else if message == "ACK" {
                println!(
                    "[RECEIVER 🤝{}] Recv: '{}' from:{} [{}] ",
                    self.id,
                    message,
                    from,
                    current_time()
                );
                *self.got_ack.0.lock().unwrap() = true;
                self.got_ack.1.notify_all();
                for handle in self.handles.drain(..) {
                    handle.join().expect("Failted to join a thread");
                }
                println!("[RECEIVER {}⏰️] Waiting a new  Message", self.id);
            }
        }
    }
}
