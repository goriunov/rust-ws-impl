use mio::tcp::{TcpListener, TcpStream};
use mio::*;

use slab::*;

use std;
use std::io::{ErrorKind, Read, Write};

pub struct SocketServer {
    is_set: bool,
    listener: TcpListener,
    clients: Slab<TcpStream>,
}

impl SocketServer {
    const TOKEN: Token = Token(0);
    pub fn new(addr: std::net::SocketAddr) -> SocketServer {
        let listener = TcpListener::bind(&addr).expect("Could not bind server");
        SocketServer {
            listener,
            clients: Slab::with_capacity(2048),
            is_set: false,
        }
    }

    pub fn start(&mut self) {
        let poll = Poll::new().unwrap();

        poll.register(
            &self.listener,
            SocketServer::TOKEN,
            Ready::readable(),
            PollOpt::edge(),
        ).unwrap();

        let mut events = Events::with_capacity(1024);

        loop {
            poll.poll(&mut events, None).unwrap();

            for e in &events {
                let token = e.token();
                let readiness = e.readiness();

                match token {
                    SocketServer::TOKEN => {
                        if readiness.is_readable() {
                            let (mut sock, _addr) = self.listener.accept().unwrap();

                            // wee need to put fake stuff in to the slab in position 0
                            if !self.is_set {
                                {
                                    let entry = self.clients.vacant_entry();
                                    let token: usize = entry.key().into();

                                    if token == 0 {
                                        let sokce_clone = sock.try_clone().unwrap();
                                        entry.insert(sokce_clone);
                                        self.is_set = true;
                                    }
                                }
                            }

                            let entry = self.clients.vacant_entry();
                            let token = entry.key().into();

                            poll.register(
                                &sock,
                                token,
                                Ready::readable() | Ready::writable(),
                                PollOpt::edge() | PollOpt::oneshot(),
                            ).unwrap();

                            entry.insert(sock);
                        }
                    }
                    Token(_) => {
                        let mut client = self.clients.remove(token.into());
                        let mut done_read = false;
                        if readiness.is_readable() {
                            let mut buf = [0; 1024];
                            let mut new_vec = Vec::with_capacity(1024);
                            let die = loop {
                                match client.read(&mut buf) {
                                    Ok(0) => break true,
                                    Ok(n) => {
                                        // println!("{:#?}", std::str::from_utf8(&buf).unwrap());
                                        new_vec.extend_from_slice(&buf[..n]);
                                    }
                                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => break false,
                                    Err(_e) => break false,
                                }
                            };

                            if die {
                                println!("killing socket: {:?}", token);
                                continue;
                            }

                            let buf_len = new_vec.len();
                            if buf_len > 3 && &new_vec[buf_len - 4..] == b"\r\n\r\n" {
                                println!("{:#?}", std::str::from_utf8(&new_vec).unwrap());
                                done_read = true;
                            }
                        }
                        let mut done_write = false;
                        if readiness.is_writable() && done_read {
                            let write_buf = b"HTTP/1.1 200 OK\r\nServer: HttpMio\r\n\r\n".to_vec();

                            loop {
                                match client.write(&write_buf[..]) {
                                    Ok(0) => break,
                                    Ok(n) => {
                                        done_write = true;
                                        // need to handle state of the data
                                        // break;
                                        // socket.bytes_written += n;
                                    }
                                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => break,
                                    Err(e) => {
                                        panic!("{:?}", e);
                                        // break;
                                    }
                                }
                            }
                            println!("{}", done_write);
                        }

                        if !done_write {
                            let entry = self.clients.vacant_entry();
                            let token = entry.key().into();
                            poll.reregister(
                                &client,
                                token,
                                Ready::readable() | Ready::writable(),
                                PollOpt::edge() | PollOpt::oneshot(),
                            ).unwrap();

                            entry.insert(client);
                        }
                    }
                }
            }
        }
    }
}
