extern crate socket;

use socket::server::SocketServer;

fn main() {
    let mut server = SocketServer::new("127.0.0.1:3000".parse().unwrap());
    server.start();
    println!("Hello world");
}
