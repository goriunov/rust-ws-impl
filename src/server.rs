use mio;
use mio::tcp::{Shutdown, TcpStream};
use mio::*;

use std;
use std::io::ErrorKind;
use std::io::{Read, Write};

use fnv::FnvHashMap;

pub struct WebsocketServer {
    server: mio::net::TcpListener,
    on_open: fn(i32),
    on_message: fn([u8; 2048]),
    clients: FnvHashMap<Token, TcpStream>,
}

impl WebsocketServer {
    const token: Token = Token(0);

    pub fn new(addr: std::net::SocketAddr) -> WebsocketServer {
        let server = mio::net::TcpListener::bind(&addr).expect("Could not bind server");
        WebsocketServer {
            server,
            on_open: |_| {},
            on_message: |_| {},
            clients: FnvHashMap::default(),
        }
    }

    pub fn start(&mut self) {
        let poll = mio::Poll::new().unwrap();
        let mut count = 0;
        poll.register(
            &self.server,
            WebsocketServer::token,
            Ready::readable(),
            PollOpt::edge(),
        ).unwrap();

        let mut events = Events::with_capacity(1024);

        loop {
            poll.poll(&mut events, None).unwrap();
            for e in &events {
                let token = e.token();

                match token {
                    WebsocketServer::token => loop {
                        match self.server.accept() {
                            Ok((socket, _)) => {
                                count += 1;
                                let new_token = Token(count);

                                poll.register(
                                    &socket,
                                    new_token,
                                    Ready::readable(),
                                    PollOpt::edge(),
                                ).unwrap();

                                self.clients.insert(new_token, socket);
                            }
                            Err(ref e) if e.kind() == ErrorKind::WouldBlock => break,
                            Err(_e) => println!("Server error"),
                        }
                    },
                    Token(_) => {
                        let mut buf = [0; 2048];

                        let stream_close = loop {
                            let mut client = self.clients.get_mut(&token).unwrap();
                            match client.read(&mut buf) {
                                Ok(0) => break true,
                                Ok(_) => break false,
                                Err(ref e) if e.kind() == ErrorKind::WouldBlock => break false,
                                Err(ref e) if e.kind() == ErrorKind::ConnectionReset => break true,
                                Err(_e) => break true,
                            }
                        };

                        if stream_close {
                            let client = self.clients.remove(&token).unwrap();
                            client.shutdown(Shutdown::Both).unwrap();
                            continue;
                        }


                        (self.on_message)(buf);
                    }
                }
            }
        }
    }

    pub fn on_open(&mut self, calback: fn(i32)) {
        self.on_open = calback;
    }

    pub fn on_message(&mut self, calback: fn([u8; 2048])) {
        self.on_message = calback;
    }
}
