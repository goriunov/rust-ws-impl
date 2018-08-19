extern crate fnv;
extern crate mio;

use std::io::{Read, Write};

use fnv::FnvHashMap;
use mio::tcp::{Shutdown, TcpListener, TcpStream};
use mio::*;

// const SERVER: Token = Token(0);

struct WebSocketClient {
    socket: TcpStream,
    interest: Ready,
    read_buf: Vec<u8>,
    hung: u8,
}

impl WebSocketClient {
    fn new(socket: TcpStream) -> Self {
        WebSocketClient {
            socket,
            hung: 0,
            interest: Ready::readable(),
            read_buf: Vec::with_capacity(1024),
        }
    }

    fn read(&mut self) {
        let mut buf = [0; 1024];

        loop {
            match self.socket.read(&mut buf) {
                Ok(0) => {
                    // record one hung from the client
                    self.hung += 1;
                    break;
                }
                Ok(n) => {
                    self.hung = 0;
                    self.read_buf.extend_from_slice(&buf[..n]);
                    println!("{:?}", std::str::from_utf8(&self.read_buf).unwrap());
                    // break;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => {
                    println!("Error while reading socket: {:?}", e);
                    return;
                }
            }
        }
    }
}

struct WebsocketServer {
    clients: FnvHashMap<Token, WebSocketClient>,
    server: TcpListener,
    token: Token,
    counter: usize,
}

impl WebsocketServer {
    fn new(addr: std::net::SocketAddr) -> Self {
        let server = TcpListener::bind(&addr).expect("Could not bind to port");

        WebsocketServer {
            token: Token(0),
            server,
            counter: 0,
            clients: FnvHashMap::default(),
        }
    }

    fn start(&mut self) {
        let poll = mio::Poll::new().unwrap();

        poll.register(&self.server, self.token, Ready::readable(), PollOpt::edge())
            .unwrap();

        let mut events = Events::with_capacity(1024);
        println!("Server is running");

        loop {
            poll.poll(&mut events, None).unwrap();

            for event in events.iter() {
                let readiness = event.readiness();
                match event.token() {
                    Token(0) => match self.server.accept() {
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
                        let mut hungs;
                        {
                            hungs = self.clients.get(&token).unwrap().hung;
                        }

                        if hungs >= 5 {
                            let client = self.clients.remove(&token).unwrap();
                            client.socket.shutdown(Shutdown::Both).unwrap();
                            poll.deregister(&client.socket).unwrap();
                        } else if readiness.is_readable() {
                            let mut client = self.clients.get_mut(&token).unwrap();
                            client.read();
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

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let mut server = WebsocketServer::new(addr);
    server.start();
}
