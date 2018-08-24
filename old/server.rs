use client::WebSocketClient;
use fnv::FnvHashMap;
use std;

use mio;
use mio::tcp::{Shutdown, TcpListener};
use mio::*;

pub struct WebsocketServer {
    server: TcpListener,
    counter: usize,
    clients: FnvHashMap<Token, WebSocketClient>,
}

impl WebsocketServer {
    const TOKEN: Token = Token(0);

    pub fn new(addr: std::net::SocketAddr) -> Self {
        let server = TcpListener::bind(&addr).expect("Could not bind to port");

        WebsocketServer {
            server,
            counter: 0,
            clients: FnvHashMap::default(),
        }
    }

    pub fn start(&mut self) {
        let poll = mio::Poll::new().unwrap();

        poll.register(
            &self.server,
            WebsocketServer::TOKEN,
            Ready::readable(),
            PollOpt::edge(),
        ).unwrap();

        let mut events = Events::with_capacity(1024);
        println!("Server is running on {}", self.server.local_addr().unwrap());

        loop {
            poll.poll(&mut events, None).unwrap();

            for event in events.iter() {
                let readiness = event.readiness();
                match event.token() {
                    WebsocketServer::TOKEN => match self.server.accept() {
                        Ok((socket, _)) => {
                            self.counter += 1;
                            let new_token = Token(self.counter);

                            poll.register(
                                &socket,
                                new_token,
                                Ready::readable(),
                                PollOpt::edge() | PollOpt::oneshot(),
                            ).unwrap();

                            self.clients.insert(new_token, WebSocketClient::new(socket));
                        }
                        Err(e) => println!("Error during connection {}", e),
                    },
                    Token(c) => {
                        let token = Token(c);
                        let is_hanged: bool = self.clients.get(&token).unwrap().hangs > 3;

                        if is_hanged {
                            let client = self.clients.remove(&token).unwrap();
                            client.socket.shutdown(Shutdown::Both).unwrap();
                            poll.deregister(&client.socket).unwrap();
                        } else if readiness.is_writable() || readiness.is_readable() {
                            let mut client = self.clients.get_mut(&token).unwrap();

                            if readiness.is_writable() {
                                client.write()
                            } else {
                                client.read();
                            }

                            poll.reregister(
                                &client.socket,
                                token,
                                client.interest,
                                PollOpt::edge() | PollOpt::oneshot(),
                            ).unwrap();
                        }
                    }
                }
            }
        }
    }
}
