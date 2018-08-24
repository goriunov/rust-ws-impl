// extern crate base64;
// extern crate fnv;
// extern crate http_muncher;
// extern crate sha1;

// pub mod client;
// pub mod parser;
// pub mod server;

// use server::WebsocketServer;
// use std::thread;

// from here
extern crate fnv;
extern crate mio;
extern crate tokio;

// use fnv::FnvHashMap;

use std::io::{ErrorKind, Read, Write};
pub mod server;

use mio::tcp::{Shutdown, TcpListener, TcpStream};
use mio::*;

use server::WebsocketServer;

// const SERVER_TOKEN: Token = Token(0);

fn main() {
    let mut server = WebsocketServer::new("127.0.0.1:3000".parse().unwrap());

    server.on_message(|buf| print!("Got data: {}", String::from_utf8(buf.to_vec()).unwrap()));

    server.start();

    // let addr = "127.0.0.1:3000".parse().unwrap();
    // let server = TcpListener::bind(&addr).unwrap();
    // let mut clients: FnvHashMap<Token, TcpStream> = FnvHashMap::default();

    // let poll = mio::Poll::new().unwrap();

    // poll.register(&server, SERVER_TOKEN, Ready::readable(), PollOpt::level())
    //     .unwrap();

    // let mut got_messages = 0;
    // let mut count = 0;
    // let mut events = Events::with_capacity(1024);

    // loop {
    //     poll.poll(&mut events, None).unwrap();
    //     for e in &events {
    //         let token = e.token();

    //         match token {
    //             SERVER_TOKEN => {
    //                 loop {
    //                     match server.accept() {
    //                         Ok((socket, _)) => {
    //                             // println!("Accepted new connection");
    //                             count += 1;

    //                             let new_token = Token(count);

    //                             poll.register(
    //                                 &socket,
    //                                 new_token,
    //                                 Ready::readable(),
    //                                 PollOpt::edge(),
    //                             ).unwrap();

    //                             clients.insert(new_token, socket);

    //                             // println!("Insert new client, {:?}", clients);
    //                         }
    //                         Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
    //                             // println!("Block not ready");
    //                             break;
    //                         }
    //                         Err(_e) => println!("Server error"),
    //                     }
    //                 }
    //             }
    //             Token(_) => {
    //                 let mut buf = [0; 1938];
    //                 // let mut client = ;
    //                 // let mut stream_close: bool;

    //                 let stream_close = loop {
    //                     match clients.get_mut(&token).unwrap().read(&mut buf) {
    //                         Ok(0) => {
    //                             // the stream has ended for real
    //                             break true;
    //                         }
    //                         Ok(_) => {
    //                             // print!("Got data: {}", String::from_utf8(buf.to_vec()).unwrap());
    //                             break false;
    //                         }
    //                         Err(ref e) if e.kind() == ErrorKind::WouldBlock => break false,
    //                         Err(ref e) if e.kind() == ErrorKind::ConnectionReset => break true,
    //                         Err(_e) => break true,
    //                     };
    //                 };

    //                 if stream_close {
    //                     // println!("Socke is closed");
    //                     let client = clients.remove(&token).unwrap();
    //                     client.shutdown(Shutdown::Both).unwrap();
    //                     // println!("{}", clients.len());
    //                     if clients.len() == 1 {
    //                         println!("Messages {}", got_messages);
    //                         got_messages = 0;
    //                     }
    //                     continue;
    //                 }

    //                 // // respond to socket
    //                 // let response = b"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n<html><body>Hello world</body></html>\r\n";

    //                 loop {
    //                     match clients.get_mut(&token).unwrap().write(&buf[..]) {
    //                         Ok(_) => {
    //                             // println!("Response sent");
    //                             break;
    //                         }
    //                         Err(e) => {
    //                             println!("Failed sending response: {}", e);
    //                             break;
    //                         }
    //                     }
    //                 }
    //                 got_messages += 1;
    //             }
    //         }
    //     }
    // }
}
