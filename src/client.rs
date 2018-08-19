use base64::encode;
use fnv::FnvHashMap;
use sha1;

use http_muncher::Parser;
use parser::HttpHeaders;

use std;
use std::io::{Read, Write};

use mio::tcp::TcpStream;
use mio::*;

fn generate_key(key: &String) -> String {
    let mut m = sha1::Sha1::new();
    m.update(key.as_bytes());
    m.update("258EAFA5-E914-47DA-95CA-C5AB0DC85B11".as_bytes());
    encode(&m.digest().bytes()).to_string()
}

pub enum WebsocketState {
    AwaitingHandshake,
    HandshakeResponse,
    Connected,
    Closed,
}

pub struct WebSocketClient {
    pub state: WebsocketState,
    pub socket: TcpStream,
    pub interest: Ready,
    pub read_buf: Vec<u8>,
    pub hangs: u8,
    pub headers: FnvHashMap<String, String>,
}

impl WebSocketClient {
    pub fn new(socket: TcpStream) -> Self {
        WebSocketClient {
            socket,
            hangs: 0,
            state: WebsocketState::AwaitingHandshake,
            headers: FnvHashMap::default(),
            interest: Ready::readable(),
            read_buf: Vec::with_capacity(1024),
        }
    }

    pub fn write(&mut self) {
        let response_key = generate_key(&self.headers.get("Sec-WebSocket-Key").unwrap());
        let response = format!(
            "HTTP/1.1 101 Switching Protocols\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Accept: {}\r\n\
             Upgrade: websocket\r\n\r\n",
            response_key
        );

        println!("{}", response);

        self.socket.write(response.as_bytes()).unwrap();
        self.state = WebsocketState::Connected;
        self.interest.remove(Ready::writable());
        self.interest.insert(Ready::readable());
    }

    pub fn read(&mut self) {
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
                        let mut http_headers = HttpHeaders::new();

                        parser.parse(&mut http_headers, &self.read_buf[..]);
                        self.headers = http_headers.get_all_headers();

                        // print all headers
                        println!("{:#?}", self.headers);

                        // check if we have upgrade request
                        if parser.is_upgrade() {
                            self.state = WebsocketState::HandshakeResponse;
                            self.interest.remove(Ready::readable());
                            self.interest.insert(Ready::writable());
                        }
                        break;
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => {
                    println!("Error while reading socket: {:?}", e);
                    return;
                }
            }
        }
    }
}
