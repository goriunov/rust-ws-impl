use mio::tcp::{Shutdown, TcpListener, TcpStream};
use std;
use std::io::{ErrorKind, Read, Write};

pub struct ReadWrite {
    pub read_buf: Vec<u8>,
    pub write_buf: Vec<u8>,
    pub done_read: bool,
    pub done_write: bool,
    pub written_bytes: usize,
}

pub struct SocketClient {
    pub stream: TcpStream,
    pub read_write: ReadWrite,
}

impl SocketClient {
    pub fn new(stream: TcpStream) -> SocketClient {
        SocketClient {
            stream,
            read_write: ReadWrite {
                read_buf: Vec::with_capacity(2048),
                write_buf: Vec::with_capacity(2048),
                done_read: false,
                done_write: false,
                written_bytes: 0,
            },
        }
    }

    pub fn read_handshake(&mut self) -> bool {
        let mut buf = [0; 256];

        let socket_closed = loop {
            match self.stream.read(&mut buf) {
                Ok(0) => break true,
                Ok(n) => {
                    self.read_write.read_buf.extend_from_slice(&buf[..n]);
                    // println!("{:#?}", std::str::from_utf8(&buf).unwrap())
                    // socket.read_buf.extend_from_slice(&buf[..n]);
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => break false,
                Err(_) => break false,
            }
        };

        socket_closed
    }

    pub fn write_handshake(&mut self) {
        loop {
            match self
                .stream
                .write(&self.read_write.write_buf[self.read_write.written_bytes..])
            {
                Ok(0) => break,
                Ok(n) => {
                    self.read_write.written_bytes += n;
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(e) => {
                    // panic!("{:?}", e);
                    break;
                }
            }
        }
    }

    pub fn terminate(&mut self) {
        match self.stream.shutdown(Shutdown::Both) {
            Ok(_) => {}
            Err(_e) => {}
        }
    }
}
