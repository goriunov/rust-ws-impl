// // extern crate base64;
// // extern crate fnv;
// // extern crate http_muncher;
// // extern crate sha1;

// // pub mod client;
// // pub mod parser;
// // pub mod server;

// // use server::WebsocketServer;
// // use std::thread;

// // from here
// extern crate fnv;
// extern crate mio;

// pub mod client;
// pub mod server;

// // use client::WebSocket;
// use server::Handlers;
// use server::WebSocketServer;

// // use mio;
// use mio::tcp::{Shutdown, TcpListener, TcpStream};
use mio::*;

// // use std;
// use std::io::ErrorKind;

// use client::WebSocket;
// use std::io::{Read, Write};

// use fnv::FnvHashMap;

// const SERVER_TOKEN: Token = Token(0);

// // extern crate mini_http;

// // fn run() -> Result<(), Box<std::error::Error>> {
// //     mini_http::Server::new("127.0.0.1:3000")?
// //         .tcp_nodelay(true)
// //         .start(|_req| {
// //             mini_http::Response::builder()
// //                 .status(200)
// //                 .body(b"Hello!\n".to_vec())
// //                 .unwrap()
// //         })?;
// //     Ok(())
// // }

// fn main() {
//     // if let Err(e) = run() {
//     //     eprintln!("Error: {}", e);
//     // }
//     let mut server = WebSocketServer::new("127.0.0.1:3000".parse().unwrap());

//     server.on_message(|socket, buf| {
//         // socket.write();
//         print!("Got data: {}", String::from_utf8(buf.to_vec()).unwrap())
//     });

//     server.start();

//     // let addr = "127.0.0.1:3000".parse().unwrap();
//     // let server = TcpListener::bind(&addr).unwrap();
//     // let mut clients: FnvHashMap<Token, TcpStream> = FnvHashMap::default();

//     // let poll = mio::Poll::new().unwrap();

//     // poll.register(&server, SERVER_TOKEN, Ready::readable(), PollOpt::level())
//     //     .unwrap();

//     // let mut got_messages = 0;
//     // let mut count = 0;
//     // let mut events = Events::with_capacity(1024);

//     // loop {
//     //     poll.poll(&mut events, None).unwrap();
//     //     for e in &events {
//     //         let token = e.token();

//     //         match token {
//     //             SERVER_TOKEN => {
//     //                 loop {
//     //                     match server.accept() {
//     //                         Ok((socket, _)) => {
//     //                             // println!("Accepted new connection");
//     //                             count += 1;

//     //                             let new_token = Token(count);

//     //                             poll.register(
//     //                                 &socket,
//     //                                 new_token,
//     //                                 Ready::readable(),
//     //                                 PollOpt::edge(),
//     //                             ).unwrap();

//     //                             clients.insert(new_token, socket);

//     //                             // println!("Insert new client, {:?}", clients);
//     //                         }
//     //                         Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
//     //                             // println!("Block not ready");
//     //                             break;
//     //                         }
//     //                         Err(_e) => println!("Server error"),
//     //                     }
//     //                 }
//     //             }
//     //             Token(_) => {
//     //                 let mut buf = [0; 1938];
//     //                 // let mut client = ;
//     //                 // let mut stream_close: bool;

//     //                 let stream_close = loop {
//     //                     match clients.get_mut(&token).unwrap().read(&mut buf) {
//     //                         Ok(0) => {
//     //                             // the stream has ended for real
//     //                             break true;
//     //                         }
//     //                         Ok(_) => {
//     //                             // print!("Got data: {}", String::from_utf8(buf.to_vec()).unwrap());
//     //                             break false;
//     //                         }
//     //                         Err(ref e) if e.kind() == ErrorKind::WouldBlock => break false,
//     //                         Err(ref e) if e.kind() == ErrorKind::ConnectionReset => break true,
//     //                         Err(_e) => break true,
//     //                     };
//     //                 };

//     //                 if stream_close {
//     //                     // println!("Socke is closed");
//     //                     let client = clients.remove(&token).unwrap();
//     //                     client.shutdown(Shutdown::Both).unwrap();
//     //                     // println!("{}", clients.len());
//     //                     if clients.len() == 1 {
//     //                         println!("Messages {}", got_messages);
//     //                         got_messages = 0;
//     //                     }
//     //                     continue;
//     //                 }

//     //                 // // respond to socket
//     //                 let response = b"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n<html><body>Hello world</body></html>\r\n";

//     //                 loop {
//     //                     match clients.get_mut(&token).unwrap().write(&response[..]) {
//     //                         Ok(_) => {
//     //                             // println!("Response sent");
//     //                             break;
//     //                         }
//     //                         Err(e) => {
//     //                             println!("Failed sending response: {}", e);
//     //                             break;
//     //                         }
//     //                     }
//     //                 }
//     //                 got_messages += 1;
//     //             }
//     //         }
//     //     }
//     // }
// }

extern crate mio;
extern crate slab;

use mio::net::TcpListener;
use std::io::{self, Read, Write};
use std::time;
extern crate fnv;
use fnv::FnvHashMap;

enum Socket {
    Listener {
        listener: mio::net::TcpListener,
    },
    Stream {
        stream: mio::net::TcpStream,
        read_buf: Vec<u8>,
        done_read: bool,
        write_buf: Vec<u8>,
        bytes_written: usize,
    },
}

impl Socket {
    fn new_listener(l: mio::net::TcpListener) -> Self {
        Socket::Listener { listener: l }
    }
    fn new_stream(s: mio::net::TcpStream) -> Self {
        Socket::Stream {
            stream: s,
            read_buf: Vec::with_capacity(1024),
            done_read: false,
            write_buf: Vec::with_capacity(1024),
            bytes_written: 0,
        }
    }
    fn continued_stream(
        stream: mio::net::TcpStream,
        read_buf: Vec<u8>,
        done_read: bool,
        write_buf: Vec<u8>,
        bytes_written: usize,
    ) -> Self {
        Socket::Stream {
            stream,
            read_buf,
            done_read,
            write_buf,
            bytes_written,
        }
    }
}

fn run() -> Result<(), Box<::std::error::Error>> {
    let mut sockets = slab::Slab::with_capacity(1024);
    let addr = "127.0.0.1:3000".parse()?;
    let server = TcpListener::bind(&addr)?;

    let poll = mio::Poll::new()?;
    {
        let entry = sockets.vacant_entry();
        let server_token = entry.key().into();
        poll.register(
            &server,
            server_token,
            mio::Ready::readable(),
            mio::PollOpt::edge(),
        )?;
        entry.insert(Socket::new_listener(server));
    }

    println!("** Listening on {} **", addr);

    let mut events = mio::Events::with_capacity(1024);

    loop {
        poll.poll(&mut events, None)?;
        for e in &events {
            let token = e.token();

            match sockets.remove(token.into()) {
                Socket::Listener { listener } => {
                    let readiness = e.readiness();
                    // println!("listener, {:?}, {:?}", token, readiness);
                    if readiness.is_readable() {
                        // println!("handling listener is readable");
                        let (mut sock, addr) = listener.accept()?;
                        // println!("opened socket to: {:?}", addr);

                        // register the newly opened socket
                        let entry = sockets.vacant_entry();
                        let token = entry.key().into();
                        poll.register(&sock, token, mio::Ready::readable(), mio::PollOpt::edge())?;
                        entry.insert(Socket::new_stream(sock));
                    }
                    // reregister listener
                    let entry = sockets.vacant_entry();
                    let token = entry.key().into();
                    poll.reregister(
                        &listener,
                        token,
                        mio::Ready::readable(),
                        mio::PollOpt::edge(),
                    )?;
                    entry.insert(Socket::new_listener(listener));
                }
                Socket::Stream {
                    mut stream,
                    mut read_buf,
                    mut done_read,
                    mut write_buf,
                    mut bytes_written,
                } => {
                    let readiness = e.readiness();
                    // println!("stream, {:?}, {:?}, done_read: {:?}", token, readiness, done_read);
                    if readiness.is_readable() {
                        let mut buf = [0; 256];
                        let die = loop {
                            match stream.read(&mut buf) {
                                Ok(0) => break true,
                                Ok(n) => {
                                    read_buf.extend_from_slice(&buf[..n]);
                                }
                                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break false,
                                Err(e) => {
                                    panic!("{:?}", e);
                                    break false;
                                }
                            }
                        };
                        if die {
                            println!("killing socket: {:?}", token);
                            continue;
                        }
                        // TODO: Parse headers/body for real
                        //       this only works for requests without a body
                        let buf_len = read_buf.len();
                        if buf_len > 3 && &read_buf[buf_len - 4..] == b"\r\n\r\n" {
                            println!("{:?}", std::str::from_utf8(&read_buf)?);
                            done_read = true;
                        }
                    }
                    let mut done_write = false;
                    if readiness.is_writable() && done_read {
                        println!("handling stream is done reading and is writable");
                        if write_buf.is_empty() {
                            println!("echo: {:?}", std::str::from_utf8(&read_buf)?);
                            write_buf = b"HTTP/1.1 200 OK\r\nServer: HttpMio\r\n\r\n".to_vec();
                            write_buf.extend_from_slice(&read_buf);
                        }
                        loop {
                            match stream.write(&write_buf[bytes_written..]) {
                                Ok(0) => break,
                                Ok(n) => {
                                    bytes_written += n;
                                }
                                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                                Err(e) => {
                                    panic!("{:?}", e);
                                    // break;
                                }
                            }
                        }
                        done_write = write_buf.len() == bytes_written;
                    }
                    
                    if !done_write {
                        // println!("Done writing");
                        // we're not done with this socket yet
                        // TODO: handle disconnected clients
                        // reregister stream
                        let entry = sockets.vacant_entry();
                        let token = entry.key().into();
                        poll.reregister(
                            &stream,
                            token,seSlotMap
                            mio::Ready::readable() | mio::Ready::writable(),
                            mio::PollOpt::edge(),
                        )?;
                        entry.insert(Socket::continued_stream(
                            stream,
                            read_buf,
                            done_read,
                            write_buf,
                            bytes_written,
                        ));
                    }
                }
            }
        }
    }

    Ok(())
}

const TOKEN: Token = Token(0);

struct WebSocket {
    // Client {
    stream: mio::net::TcpStream,
    read_buf: Vec<u8>,
    done_read: bool,
    write_buf: Vec<u8>,
    bytes_written: usize,
    // },
    // Server {
    //     listener: mio::tcp::TcpListener,
    // },
}
// struct WebSocket {
//     stream: mio::net::TcpStream,
//     read_buf: Vec<u8>,
//     done_read: bool,
//     write_buf: Vec<u8>,
//     bytes_written: usize,
// }

impl WebSocket {
    fn new_client(stream: mio::net::TcpStream) -> WebSocket {
        WebSocket {
            stream,
            read_buf: Vec::with_capacity(1024),
            done_read: false,
            write_buf: Vec::with_capacity(1024),
            bytes_written: 0,
        }
    }
    // fn new_server(listener: mio::net::TcpListener) -> WebSocket {
    //     WebSocket::Server { listener }
    // }
    fn continued_stream(
        stream: mio::net::TcpStream,
        read_buf: Vec<u8>,
        done_read: bool,
        write_buf: Vec<u8>,
        bytes_written: usize,
    ) -> Self {
        WebSocket {
            stream,
            read_buf,
            done_read,
            write_buf,
            bytes_written,
        }
    }
}

fn main() {
    let mut sockets: slab::Slab<WebSocket> = slab::Slab::with_capacity(2048);
    let addr = "127.0.0.1:3000".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();
    let poll = mio::Poll::new().unwrap();

    // {
    //     let entry = sockets.vacant_entry();
    //     let server_token = entry.key().into();

    poll.register(
        &listener,
        Token(100000),
        mio::Ready::readable(),
        mio::PollOpt::edge(),
    ).unwrap();

    //     entry.insert(WebSocket::new_server(listener));
    // }

    let mut events = mio::Events::with_capacity(1024);

    loop {
        poll.poll(&mut events, None).unwrap();

        for e in &events {
            let token = e.token();
            let readiness = e.readiness();

            match token {
                Token(100000) => {
                    if readiness.is_readable() {
                        let (mut sock, _addr) = listener.accept().unwrap();
                        let entry = sockets.vacant_entry();
                        let token = entry.key().into();
                        poll.register(
                            &sock,
                            token,
                            mio::Ready::readable(),
                            mio::PollOpt::edge() | PollOpt::oneshot(),
                        ).unwrap();
                        entry.insert(WebSocket::new_client(sock));
                    }
                }
                Token(_) => {
                    let mut socket = sockets.remove(token.into());
                    if readiness.is_readable() {
                        let mut buf = [0; 256];
                        let die = loop {
                            match socket.stream.read(&mut buf) {
                                Ok(0) => break true,
                                Ok(n) => {
                                    socket.read_buf.extend_from_slice(&buf[..n]);
                                }
                                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break false,
                                Err(e) => {
                                    panic!("{:?}", e);
                                    break false;
                                }
                            }
                        };
                        if die {
                            println!("killing socket: {:?}", token);
                            continue;
                        }
                        // TODO: Parse headers/body for real
                        //       this only works for requests without a body
                        let buf_len = socket.read_buf.len();
                        if buf_len > 3 && &socket.read_buf[buf_len - 4..] == b"\r\n\r\n" {
                            println!("{:?}", std::str::from_utf8(&socket.read_buf).unwrap());
                            socket.done_read = true;
                        }
                    }
                    let mut done_write = false;
                    if readiness.is_writable() && socket.done_read {
                        println!("handling stream is done reading and is writable");
                        if socket.write_buf.is_empty() {
                            println!("echo: {:?}", std::str::from_utf8(&socket.read_buf).unwrap());
                            socket.write_buf =
                                b"HTTP/1.1 200 OK\r\nServer: HttpMio\r\n\r\n".to_vec();
                            socket.write_buf.extend_from_slice(&socket.read_buf);
                        }
                        loop {
                            match socket
                                .stream
                                .write(&socket.write_buf[socket.bytes_written..])
                            {
                                Ok(0) => break,
                                Ok(n) => {
                                    socket.bytes_written += n;
                                }
                                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                                Err(e) => {
                                    panic!("{:?}", e);
                                    // break;
                                }
                            }
                        }
                        done_write = socket.write_buf.len() == socket.bytes_written;
                    }

                    if !done_write {
                        // println!("Done writing");
                        // we're not done with this socket yet
                        // TODO: handle disconnected clients
                        // reregister stream
                        let entry = sockets.vacant_entry();
                        let token = entry.key().into();
                        // let new_token = Token(2132132132);
                        poll.reregister(
                            &socket.stream,
                            token,
                            mio::Ready::readable() | mio::Ready::writable(),
                            mio::PollOpt::edge() | PollOpt::oneshot(),
                        ).unwrap();
                        entry.insert(WebSocket::continued_stream(
                            socket.stream,
                            socket.read_buf,
                            socket.done_read,
                            socket.write_buf,
                            socket.bytes_written,
                        ));
                    }
                }
            };
            // WebSocket::Server { listener } => {
            //     if readiness.is_readable() {
            //         let (mut sock, _addr) = listener.accept().unwrap();
            //         let entry = sockets.vacant_entry();
            //         let token = entry.key().into();
            //         poll.register(&sock, token, mio::Ready::readable(), mio::PollOpt::edge())
            //             .unwrap();
            //         entry.insert(WebSocket::new_client(sock));
            //     }
            //     let entry = sockets.vacant_entry();
            //     let token = entry.key().into();
            //     poll.reregister(
            //         &listener,
            //         token,
            //         mio::Ready::readable(),
            //         mio::PollOpt::edge(),
            //     ).unwrap();
            //     entry.insert(WebSocket::new_server(listener));
            // }
            // WebSocket::Client {
            //     mut stream,
            //     mut read_buf,
            //     mut done_read,
            //     mut write_buf,
            //     mut bytes_written,
            // } => {
            //     if readiness.is_readable() {
            //         let mut buf = [0; 256];
            //         let die = loop {
            //             match stream.read(&mut buf) {
            //                 Ok(0) => break true,
            //                 Ok(n) => {
            //                     read_buf.extend_from_slice(&buf[..n]);
            //                 }
            //                 Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break false,
            //                 Err(e) => {
            //                     panic!("{:?}", e);
            //                     break false;
            //                 }
            //             }
            //         };
            //         if die {
            //             println!("killing socket: {:?}", token);
            //             continue;
            //         }
            //         // TODO: Parse headers/body for real
            //         //       this only works for requests without a body
            //         let buf_len = read_buf.len();
            //         if buf_len > 3 && &read_buf[buf_len - 4..] == b"\r\n\r\n" {
            //             println!("{:?}", std::str::from_utf8(&read_buf).unwrap());
            //             done_read = true;
            //         }
            //     }
            //     let mut done_write = false;
            //     if readiness.is_writable() && done_read {
            //         println!("handling stream is done reading and is writable");
            //         if write_buf.is_empty() {
            //             println!("echo: {:?}", std::str::from_utf8(&read_buf).unwrap());
            //             write_buf = b"HTTP/1.1 200 OK\r\nServer: HttpMio\r\n\r\n".to_vec();
            //             write_buf.extend_from_slice(&read_buf);
            //         }
            //         loop {
            //             match stream.write(&write_buf[bytes_written..]) {
            //                 Ok(0) => break,
            //                 Ok(n) => {
            //                     bytes_written += n;
            //                 }
            //                 Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
            //                 Err(e) => {
            //                     panic!("{:?}", e);
            //                     // break;
            //                 }
            //             }
            //         }
            //         done_write = write_buf.len() == bytes_written;
            //     }

            //     if !done_write {
            //         // println!("Done writing");
            //         // we're not done with this socket yet
            //         // TODO: handle disconnected clients
            //         // reregister stream
            //         let entry = sockets.vacant_entry();
            //         let token = entry.key().into();
            //         poll.reregister(
            //             &stream,
            //             token,
            //             mio::Ready::readable() | mio::Ready::writable(),
            //             mio::PollOpt::edge(),
            //         ).unwrap();
            //         entry.insert(WebSocket::continued_stream(
            //             stream,
            //             read_buf,
            //             done_read,
            //             write_buf,
            //             bytes_written,
            //         ));
            //     }
            // }
        }
    }
}
