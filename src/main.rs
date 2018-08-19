extern crate fnv;
extern crate mio;

use std::io::{Read, Write};

use fnv::FnvHashMap;
use mio::net::{TcpListener, TcpStream};
use mio::*;

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

const SERVER: Token = Token(0);

fn main() {
    let mut counter = 0;
    let addr = "127.0.0.1:3000".parse().unwrap();
    let server = TcpListener::bind(&addr).expect("Could not bind to port");

    let poll = mio::Poll::new().unwrap();

    let mut clients: FnvHashMap<Token, WebSocketClient> = FnvHashMap::default();

    poll.register(&server, SERVER, Ready::readable(), PollOpt::edge())
        .unwrap();

    let mut events = Events::with_capacity(1024);

    loop {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            let readness = event.readiness();
            match event.token() {
                SERVER => match server.accept() {
                    Ok((socket, _)) => {
                        counter += 1;
                        let new_token = Token(counter);
                        clients.insert(new_token, WebSocketClient::new(socket));

                        poll.register(
                            &clients[&new_token].socket,
                            new_token,
                            Ready::readable(),
                            PollOpt::edge() | PollOpt::oneshot(),
                        ).unwrap();
                    }
                    Err(e) => {
                        println!("Accept error: {}", e);
                    }
                },
                Token(c) => {
                    let token = Token(c);
                    let mut hungs;
                    // get status of the hungs
                    {
                        hungs = clients.get(&token).unwrap().hung;
                    }
                    // handle if client hungs on loop
                    if hungs >= 5 {
                        let client = clients.remove(&token).unwrap();
                        poll.deregister(&client.socket).unwrap();
                    } else if readness.is_readable() {
                        let mut client = clients.get_mut(&token).unwrap();
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
