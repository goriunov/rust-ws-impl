extern crate fnv;
extern crate mio;

use std::io::{Read, Write};

use fnv::FnvHashMap;
use mio::tcp::{Shutdown, TcpListener, TcpStream};
use mio::*;

struct WebSocketClient {
    socket: TcpStream,
    interest: Ready,
    read_buf: Vec<u8>,
    hangs: u8,
}

impl WebSocketClient {
    fn new(socket: TcpStream) -> Self {
        WebSocketClient {
            socket,
            hangs: 0,
            interest: Ready::readable(),
            read_buf: Vec::with_capacity(1024),
        }
    }

    fn read(&mut self) {
        let mut buf = [0; 1024];

        loop {
            match self.socket.read(&mut buf) {
                Ok(0) => {
                    self.hangs += 1;
                    break;
                }
                Ok(n) => {
                    self.hangs = 0;
                    self.read_buf.extend_from_slice(&buf[..n]);
                    // implement header parse function
                    println!("{:?}", std::str::from_utf8(&self.read_buf).unwrap());
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // println!("Got in here with error");
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
    server: TcpListener,
    counter: usize,
    clients: FnvHashMap<Token, WebSocketClient>,
}

impl WebsocketServer {
    const TOKEN: Token = Token(0);

    fn new(addr: std::net::SocketAddr) -> Self {
        let server = TcpListener::bind(&addr).expect("Could not bind to port");

        WebsocketServer {
            server,
            counter: 0,
            clients: FnvHashMap::default(),
        }
    }

    fn start(&mut self) {
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
                        let is_hanged: bool = self.clients.get(&token).unwrap().hangs > 5;

                        if is_hanged {
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
                        } else if readiness.is_writable() {

                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let mut server = WebsocketServer::new("127.0.0.1:3000".parse().unwrap());
    server.start();
}
