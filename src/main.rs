#![allow(unused_imports)]
mod parser;
use crate::parser::{encode, redis_parse, RESPError, RedisBufSplit};
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpListener,
    sync::{Arc, Mutex},
    thread::spawn,
};

fn main() {
    println!("Logs from your program will appear here!");

    let mut keystore: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>> = Arc::new(Mutex::new(HashMap::new()));
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    let store1 = Arc::clone(&keystore);

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
                                            let key = if let RedisBufSplit::String(key) = &values[1]
                                            {
                                                println!("Key: {:?}", key);
                                                key
                                            } else {
                                                panic!("Key Issue")
                                            };
                                            let value = &values[2];
                                            let value =
                                                if let RedisBufSplit::String(value) = &values[2] {
                                                    println!("Value: {:?}", value);
                                                    key
                                                } else {
                                                    panic!("Value Issue")
                                                };
                                            println!("SET was called");
                                            let mut store1_unlocked = store1.lock().unwrap();
                                            store1_unlocked.insert(
                                                key.as_slice(&buf).to_vec(),
                                                value.as_slice(&buf).to_vec(),
                                            );
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
