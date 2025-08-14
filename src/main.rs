#![allow(unused_imports)]
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread::spawn,
};

const CR: u8 = b'\r';
const LD: u8 = b'\n';

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                spawn(move || {
                    let mut buf = [0; 64];
                    loop {
                        let bytes_read = stream.read(&mut buf).unwrap();
                        println!("Number of Bytes: {} \n Data: {:?}", bytes_read, buf);
                        parse_command_array(&mut buf);
                        if bytes_read == 0 {
                            return;
                        }
                        stream.write_all(b"+PONG\r\n").unwrap();
                    }
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

pub enum RedisCommand {
    PING,
    ECHO { data: String },
}
pub fn parse_command_array(buf: &mut [u8; 64]) -> RedisCommand {
    assert!(buf[0] == b'*');
    let array_length = buf[1];
    let bulk_string_symbol = buf[6];
    assert!(bulk_string_symbol == b'$');
    let command_length = (buf[7] - b'0') as usize;
    let command_string = String::from_utf8(buf[12..(12 + command_length)].to_vec()).unwrap();
    println!("{:?}", command_string);
    RedisCommand::PING
}
