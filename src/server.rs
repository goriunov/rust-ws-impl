use mio;
use mio::tcp::{Shutdown, TcpStream};
use mio::*;

use std;
use std::io::ErrorKind;

use client::WebSocket;
use std::io::{Read, Write};

use fnv::FnvHashMap;

pub trait Handlers {
    fn on_open(&mut self, cb: fn(&mut WebSocket));
    fn on_message(&mut self, cb: fn(&mut WebSocket, Vec<u8>));
}

pub struct WebSocketServer {
    server: mio::net::TcpListener,
    on_open: fn(&mut WebSocket),
    on_message: fn(&mut WebSocket, Vec<u8>),
    clients: FnvHashMap<Token, WebSocket>,
}

impl Handlers for WebSocketServer {
    fn on_open(&mut self, cb: fn(&mut WebSocket)) {
        self.on_open = cb;
    }
    fn on_message(&mut self, cb: fn(&mut WebSocket, Vec<u8>)) {
        self.on_message = cb;
    }
}

impl WebSocketServer {
    const TOKEN: Token = Token(0);

    pub fn new(addr: std::net::SocketAddr) -> WebSocketServer {
        let server = mio::net::TcpListener::bind(&addr).expect("Could not bind server");
        WebSocketServer {
            server,
            on_open: |_| {},
            on_message: |_, _| {},
            clients: FnvHashMap::default(),
        }
    }

    pub fn start(&mut self) {
        let poll = mio::Poll::new().unwrap();
        let mut count = 0;
        poll.register(
            &self.server,
            WebSocketServer::TOKEN,
            Ready::readable(),
            PollOpt::edge(),
        ).unwrap();

        let mut events = Events::with_capacity(1024);

        'next: loop {
            poll.poll(&mut events, None).unwrap();
            for e in &events {
                let token = e.token();
                let readiness = e.readiness();

                match token {
                    WebSocketServer::TOKEN => if readiness.is_readable() {
                        loop {
                            match self.server.accept() {
                                Ok((socket, _)) => {
                                    count += 1;
                                    let new_token = Token(count);

                                    poll.register(
                                        &socket,
                                        new_token,
                                        Ready::readable(),
                                        PollOpt::edge() | PollOpt::oneshot(),
                                    ).unwrap();

                                    self.clients.insert(new_token, WebSocket::new(socket));
                                }
                                Err(ref e) if e.kind() == ErrorKind::WouldBlock => break,
                                Err(_e) => println!("Server error"),
                            }
                        }
                    },
                    Token(_) => {
                        // set dafault value
                        let mut response: ([u8; 100], bool) = ([0; 100], false);
                        // let mut done_readign = true;
                        if readiness.is_readable() {
                            {
                                let mut client = self.clients.get_mut(&token).unwrap();
                                response = client.read();
                            }

                            let (buf, close_socket) = response;

                            if close_socket {
                                let mut client = self.clients.remove(&token).unwrap();
                                poll.deregister(&client.socket).unwrap();
                                client.terminate();
                                continue 'next;
                            }

                            // need to think about this part
                        }

                        let client = self.clients.get_mut(&token).unwrap();

                        // if client.done_read {
                        // (self.on_message)(client, client.read_buf);
                        // }
                        // let mut done_writing = true;
                        if readiness.is_writable() && client.done_read {
                            client.write_buf =
                                b"HTTP/1.1 200 OK\r\nServer: HttpMio\r\n\r\n".to_vec();
                            client.write_buf.extend_from_slice(&client.read_buf);
                            // (self.on_message)(self.clients.get_mut(&token).unwrap(), buf);
                            // println!("{:?}", client.write_buf)
                            client.write();
                        }

                        if !client.done_write {
                            // println!("Stil keep");
                            poll.reregister(
                                &client.socket,
                                token,
                                Ready::readable() | Ready::writable(),
                                PollOpt::edge() | PollOpt::oneshot(),
                            ).unwrap();
                        }

                        // if done_writing && done_readign {
                        //     let mut client = self.clients.remove(&token).unwrap();
                        //     client.terminate();
                        //     continue;
                        // }

                        // poll.reregister(
                        //     &client.socket,
                        //     token,
                        //     Ready::readable() | Ready::writable(),
                        //     PollOpt::edge(),
                        // ).unwrap();

                        // let response = b"HTTP/1.1 404 OK\r\n";
                        // client.write(response.to_vec());
                        // }/

                        // poll.reregister(
                        //     &client.socket,
                        //     token,
                        //     Ready::readable() | Ready::writable(),
                        //     PollOpt::edge() | PollOpt::oneshot(),
                        // ).unwrap();
                    }
                }
            }
        }
    }
}
