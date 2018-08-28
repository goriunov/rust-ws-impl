use mio::tcp::{TcpListener, TcpStream};
use mio::*;

use slab::*;

use std;
use std::*;

use client::SocketClient;

const SERVER_TOKEN: Token = Token(std::usize::MAX - 1);

pub struct SocketServer {
    listener: TcpListener,
    clients: Slab<SocketClient>,
    poll: Poll,
    on_open: Box<FnMut(&mut SocketClient)>,
    on_message: Box<FnMut(&mut SocketClient, Vec<u8>)>,
    on_close: Box<FnMut(&mut SocketClient)>,
}

impl SocketServer {
    pub fn on_open<CB: 'static + FnMut(&mut SocketClient)>(&mut self, cb: CB) {
        self.on_open = Box::new(cb);
    }

    pub fn on_message<CB: 'static + FnMut(&mut SocketClient, Vec<u8>)>(&mut self, cb: CB) {
        self.on_message = Box::new(cb);
    }

    pub fn on_close<CB: 'static + FnMut(&mut SocketClient)>(&mut self, cb: CB) {
        self.on_close = Box::new(cb);
    }
}

impl SocketServer {
    pub fn new(addr: std::net::SocketAddr) -> SocketServer {
        let listener = TcpListener::bind(&addr).expect("Could not bind server");
        SocketServer {
            listener,
            poll: Poll::new().unwrap(),
            clients: Slab::with_capacity(2048),
            on_open: Box::new(|_| {}),
            on_message: Box::new(|_, _| {}),
            on_close: Box::new(|_| {}),
        }
    }

    fn register_new_connection(&mut self, sock: TcpStream) {
        let entry = self.clients.vacant_entry();
        let token = entry.key().into();

        self.poll
            .register(
                &sock,
                token,
                Ready::readable() | Ready::writable(),
                PollOpt::edge() | PollOpt::oneshot(),
            )
            .expect("Could not register socket");

        let mut new_client = SocketClient::new(sock);

        // pass clinet to the user
        (self.on_open)(&mut new_client);

        entry.insert(new_client);
    }

    pub fn start(&mut self) {
        self.poll
            .register(
                &self.listener,
                SERVER_TOKEN,
                Ready::readable(),
                PollOpt::edge(),
            )
            .unwrap();

        let mut events = Events::with_capacity(1024);

        loop {
            self.poll.poll(&mut events, None).unwrap();

            'next_event: for e in &events {
                let token = e.token();
                let readiness = e.readiness();

                match token {
                    SERVER_TOKEN => {
                        if readiness.is_readable() {
                            let (mut sock, _addr) = match self.listener.accept() {
                                Ok((sock, _addr)) => (sock, _addr),
                                Err(_) => continue 'next_event,
                            };

                            sock.set_nodelay(true).expect("Could not set no delay");
                            self.register_new_connection(sock);
                        }
                    }
                    Token(_) => {
                        let mut client = self.clients.remove(token.into());

                        // fix this part from the websocket system
                        if readiness.is_readable() {
                            let stream_closed = client.read();

                            if stream_closed {
                                (self.on_close)(&mut client);
                                // terminate client and drop connection
                                client.terminate();
                                continue 'next_event;
                            }

                            // grab message form the memory and replace it with new empty array;
                            let read_buf =
                                std::mem::replace(&mut client.read_write.read_buf, Vec::new());

                            // send message to the user
                            (self.on_message)(&mut client, read_buf);
                        }

                        let entry = self.clients.vacant_entry();
                        let token = entry.key().into();
                        self.poll
                            .reregister(
                                &client.stream,
                                token,
                                Ready::readable() | Ready::writable(),
                                PollOpt::edge() | PollOpt::oneshot(),
                            )
                            .unwrap();

                        entry.insert(client);
                    }
                }
            }
        }
    }
}
