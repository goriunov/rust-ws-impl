extern crate fnv;
extern crate http_muncher;
extern crate mio;

use fnv::FnvHashMap;
use http_muncher::{Parser, ParserHandler};
use mio::tcp::{Shutdown, TcpListener, TcpStream};
use mio::*;
use std::io::{Read, Write};

struct HttpHeaders {
    headers: FnvHashMap<String, String>,
    field: String,
    value: String,
}

impl HttpHeaders {
    fn get_all_headers(self) -> FnvHashMap<String, String> {
        self.headers
    }
}

impl ParserHandler for HttpHeaders {
    fn on_header_field(&mut self, _: &mut Parser, header: &[u8]) -> bool {
        self.field = String::from_utf8(header.to_vec()).unwrap();
        true
    }
    fn on_header_value(&mut self, _: &mut Parser, value: &[u8]) -> bool {
        self.value = String::from_utf8(value.to_vec()).unwrap();
        if !self.field.is_empty() && !self.value.is_empty() {
            self.headers.insert(self.field.clone(), self.value.clone());
            // reset values
            self.field = String::new();
            self.value = String::new();
        }
        true
    }
}

struct WebSocketClient {
    socket: TcpStream,
    interest: Ready,
    read_buf: Vec<u8>,
    hangs: u8,
    headers: FnvHashMap<String, String>,
}

impl WebSocketClient {
    fn new(socket: TcpStream) -> Self {
        WebSocketClient {
            socket,
            hangs: 0,
            headers: FnvHashMap::default(),
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

                    let buf_len = self.read_buf.len();

                    // check if full headers are loaded
                    // need to write own headers parser
                    if buf_len > 3 && &self.read_buf[buf_len - 4..] == b"\r\n\r\n" {
                        let mut parser = Parser::request();
                        let mut http_headers = HttpHeaders {
                            headers: FnvHashMap::default(),
                            field: String::new(),
                            value: String::new(),
                        };

                        parser.parse(&mut http_headers, &self.read_buf[..]);
                        self.headers = http_headers.get_all_headers();

                        println!("{:#?}", self.headers);
                    }

                    // println!("{:?}", std::str::from_utf8(&self.read_buf).unwrap());
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
