use std;
use std::io::{Read, Write};

use mio::tcp::{Shutdown, TcpStream};
use mio::*;
use std::io::ErrorKind;

pub struct WebSocket {
    pub socket: TcpStream,
    pub read_buf: Vec<u8>,
    pub write_buf: Vec<u8>,
    pub done_read: bool,
    pub done_write: bool,
    pub written: usize,
}

impl WebSocket {
    pub fn new(socket: TcpStream) -> WebSocket {
        WebSocket {
            socket,
            read_buf: Vec::with_capacity(1024),
            write_buf: Vec::with_capacity(1024),
            done_read: false,
            done_write: false,
            written: 0,
        }
    }

    pub fn read(&mut self) -> ([u8; 100], bool) {
        let mut buf = [0; 100];
        let stream_close = loop {
            match self.socket.read(&mut buf) {
                Ok(0) => break true,
                Ok(n) => {
                    self.read_buf.extend_from_slice(&buf[..n]);
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => break false,
                Err(ref e) if e.kind() == ErrorKind::ConnectionReset => break true,
                Err(_e) => break true,
            }
        };
        let buf_len = self.read_buf.len();
        if buf_len > 3 && &self.read_buf[buf_len - 4..] == b"\r\n\r\n" {
            self.done_read = true;
        }
        (buf, stream_close)
    }

    pub fn write(&mut self) {
        println!("{:?}", String::from_utf8(self.write_buf.clone()).unwrap());
        loop {
            // println!("{}", )
            match self.socket.write(&self.write_buf[self.written..]) {
                Ok(0) => break,
                Ok(n) => {
                    self.written += n;
                    // println!("Wrote, {}", n);
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(_e) => break,
            }
        }

        self.done_write = self.write_buf.len() == self.written;

        // if buf.len() == self.write_b {
        //     self.write_b = 0;
        //     return true;
        // }
        // return false;
    }

    pub fn terminate(&mut self) {
        // work out error;
        match self.socket.shutdown(Shutdown::Both) {
            Ok(_) => {}
            Err(_e) => {}
        }
    }
}
