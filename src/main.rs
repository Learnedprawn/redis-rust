#![allow(unused_imports)]
mod parser;
use crate::parser::{encode, redis_parse, RESPError, RedisBufSplit};
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread::spawn,
};

// use bytes::{Bytes, BytesMut};

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted connected");
                spawn(move || {
                    let mut buf: Vec<u8> = vec![0u8; 64];
                    loop {
                        let bytes_read = stream.read(&mut buf).unwrap();
                        if bytes_read <= 0 {
                            return;
                        }
                        println!("{:?}", String::from_utf8(buf.clone()).unwrap());

                        match redis_parse(&buf, 0) {
                            Err(e) => {
                                println!("Redis Parse Error: {:?}", e);
                            }
                            Ok(result_option) => match result_option {
                                Some((pos, RedisBufSplit::Array(values))) => {
                                    println!("Position: {}, Values: {:?}", pos, values);
                                    let command = if let RedisBufSplit::String(buff) = &values[0] {
                                        println!("Command = {:?}", buff);
                                        buff
                                    } else {
                                        panic!("comand not found")
                                    };
                                    match command.as_slice(&buf) {
                                        b"PING" => stream.write_all(b"+PONG\r\n").unwrap(),
                                        b"ECHO" => {
                                            let argument = &values[1];
                                            println!("ECHO was called");
                                            let output = encode(argument, &buf);
                                            println!("Output: {:?}", output);
                                            stream.write_all(output.as_bytes()).unwrap();
                                        }
                                        b"SET" => {
                                            let key = &values[1];
                                            let value = &values[2];
                                            println!("SET was called");
                                            let output = encode(key, &buf);
                                            println!("Output: {:?}", output);
                                            stream.write_all(b"+OK\r\n").unwrap();
                                        }
                                        b"GET" => {
                                            let key = &values[1];
                                            println!("GET was called");
                                            let output = encode(key, &buf);
                                            println!("Output: {:?}", output);
                                            stream.write_all(b"+OK\r\n").unwrap();
                                        }
                                        _ => println!("Something else called"),
                                    }
                                }
                                None => println!("None value matched"),
                                _ => println!("Something other than an array was passed in"),
                            },
                        }
                    }
                });
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
    }
}

pub fn new_entry_in_hashmap(key: &RedisBufSplit, value: &RedisBufSplit) -> Result<(), ()> {
    Ok(())
}
