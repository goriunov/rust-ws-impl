use mio::tcp::{TcpListener, TcpStream};
use mio::*;

use slab::*;

use std;
use std::io::{ErrorKind, Read, Write};
use std::*;

use client::SocketClient;

pub struct SocketServer {
    listener: TcpListener,
    clients: Slab<SocketClient>,
}

impl SocketServer {
    const TOKEN: Token = Token(std::usize::MAX - 1);
    pub fn new(addr: std::net::SocketAddr) -> SocketServer {
        let listener = TcpListener::bind(&addr).expect("Could not bind server");
        SocketServer {
            listener,
            clients: Slab::with_capacity(2048),
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

        'next: loop {
            poll.poll(&mut events, None).unwrap();

            for e in &events {
                let token = e.token();
                let readiness = e.readiness();

                match token {
                    SocketServer::TOKEN => {
                        if readiness.is_readable() {
                            let (mut sock, _addr) =
                                self.listener.accept().expect("Could not get socket");

                            let entry = self.clients.vacant_entry();
                            let token = entry.key().into();

                            poll.register(&sock, token, Ready::readable(), PollOpt::edge())
                                .expect("Could not register socket");

                            entry.insert(SocketClient::new(sock));
                        }
                    }
                    Token(_) => {
                        let mut client = self.clients.remove(token.into());

                        // fix this part from the websocket system
                        if readiness.is_readable() {
                            let stream_closed = client.read_handshake();

                            if stream_closed {
                                client.terminate();
                                continue 'next;
                            }

                            // set done reading write proper header parser
                            let filled_buf = &client.read_write.read_buf;
                            let buf_len = filled_buf.len();
                            if buf_len > 3 && &filled_buf[buf_len - 4..] == b"\r\n\r\n" {
                                println!("{:#?}", str::from_utf8(&filled_buf).unwrap());
                                client.read_write.done_read = true;
                            }
                        }

                        if readiness.is_writable() && client.read_write.done_read {
                            if client.read_write.write_buf.is_empty() {
                                client.read_write.write_buf =
                                    b"HTTP/1.1 200 OK\r\nServer: HttpMio\r\n\r\n".to_vec();
                                client
                                    .read_write
                                    .write_buf
                                    .extend_from_slice(&client.read_write.read_buf);
                            }

                            client.write_handshake();

                            client.read_write.done_write = client.read_write.write_buf.len()
                                == client.read_write.written_bytes;
                        }

                        if !client.read_write.done_write {
                            let entry = self.clients.vacant_entry();
                            let token = entry.key().into();
                            poll.reregister(
                                &client.stream,
                                token,
                                Ready::readable() | Ready::writable(),
                                PollOpt::edge(),
                            ).unwrap();

                            entry.insert(client);
                        }
                    }
                }
            }
        }
    }
}
