extern crate socket;

use socket::client::SocketClient;
use socket::server::SocketServer;

use std::cell::RefCell;
use std::rc::Rc;

struct ShareState {
    count: usize,
}

impl ShareState {
    fn new() -> ShareState {
        ShareState { count: 0 }
    }
}

fn main() {
    let mut shared_state = Rc::new(RefCell::new(ShareState::new()));

    let mut server = SocketServer::new("127.0.0.1:3000".parse().unwrap());
    let mut count = 0;
    // let mut text: Vec<u8> = Vec::new();
    {
        let shared_state = shared_state.clone();
        server.on_open(move |socket| {
            println!("New client is connected");
            // socket.send_text("Hi back".to_string());
            // fields.push(socket);
            // lastConnectedClient = &mut socket;
        });
    }
    // server.o
    // server.on
    {
        let shared_state = shared_state.clone();
        server.on_message(move |socket, buf| {
            // text = buf;
            // println!("{}", std::str::from_utf8(&buf).unwrap());

            socket.send_vec(buf);
            shared_state.borrow_mut().count += 1;
            count += 1;
        });
    }
    {
        let shared_state = shared_state.clone();
        server.on_close(move |socket| {
            println!("Socket has closed");
            println!("{}", shared_state.borrow_mut().count);
        });
    }

    server.start();
    // println!("Hello world");
}
