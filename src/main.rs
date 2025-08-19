#![allow(unused_imports)]
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread::spawn,
};

const CR_LEN: usize = 1;
const LF_LEN: usize = 1;
const CR: u8 = b'\r';
const LD: u8 = b'\n';

fn main() {
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
                        let echo_command = parse_command_array(&mut buf);
                        match echo_command {
                            RedisCommand::PING => stream.write_all(b"+PONG\r\n").unwrap(),
                            RedisCommand::ECHO { data } => {
                                let response_string = &data;
                                let response_string_len = &data.len();
                                let output =
                                    format!("${}\r\n{}\r\n", response_string_len, response_string);
                                stream.write_all(output.as_bytes()).unwrap();
                            }
                        }
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
    let mut index = 0;
    assert!(buf[index] == b'*');
    index += 1; //1
    let array_length = buf[index];
    index += 1; //2
    index += CR_LEN + LF_LEN; //6
    let bulk_string_symbol = buf[index];
    assert!(bulk_string_symbol == b'$');
    index += 1; //7
    let command_length = (buf[index] - b'0') as usize;
    index += 1; //8
    index += CR_LEN + LF_LEN; //12
    let command_string = String::from_utf8(buf[index..(index + command_length)].to_vec()).unwrap();
    index += command_length; //16 if ECHO is the command
    index += CR_LEN + LF_LEN; //20
    let bulk_string_symbol = buf[index];
    assert!(bulk_string_symbol == b'$');
    index += 1; //21
    let command_argument_length = (buf[index] - b'0') as usize;
    index += 1; //8
    index += CR_LEN + LF_LEN; //12
    let command_argument_string =
        String::from_utf8(buf[index..(index + command_argument_length)].to_vec()).unwrap();
    println!("{:?}", command_argument_string);
    // assert!(command_argument_string == "hey");
    if command_string == "ECHO" {
        return RedisCommand::ECHO {
            data: command_argument_string,
        };
    }
    RedisCommand::PING
}
