extern crate base64;
extern crate fnv;
extern crate http_muncher;
extern crate mio;
extern crate sha1;

pub mod client;
pub mod parser;
pub mod server;

use server::WebsocketServer;

fn main() {
    let mut server = WebsocketServer::new("127.0.0.1:3000".parse().unwrap());
    server.start();
}
