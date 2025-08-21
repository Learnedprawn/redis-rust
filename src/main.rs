#![allow(unused_imports)]
use memchr::memchr;
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread::spawn,
};

// use bytes::{Bytes, BytesMut};

const CR_LEN: usize = 1;
const LF_LEN: usize = 1;
const CR: u8 = b'\r';
const LD: u8 = b'\n';

type RedisResult = Result<Option<(usize, RedisBufSplit)>, RESPError>;

pub enum RedisValueRef {
    String(Vec<u8>),
    Error(Vec<u8>),
    Int(i64),
    Array(Vec<RedisValueRef>),
    NullArray,
    NullBulkString,
}

#[derive(Debug)]
pub enum RESPError {
    UnexpectedEnd,
    UnknownStartingByte,
    IOError(std::io::Error),
    IntParseFailure,
    BadBulkStringSize(i64),
    BadArraySize(i64),
}
#[derive(Debug)]
struct BufSplit(usize, usize);

impl BufSplit {
    fn as_slice<'a>(&self, buf: &'a Vec<u8>) -> &'a [u8] {
        &buf[self.0..self.1]
    }
    fn len(&self) -> usize {
        self.1 - self.0 + 1
    }
}

#[derive(Debug)]
enum RedisBufSplit {
    String(BufSplit),
    Error(BufSplit),
    Int(i64),
    Array(Vec<RedisBufSplit>),
    NullArray,
    NullBulkString,
}
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
                                    // let argument = if let RedisBufSplit::String(buff) = &values[1] {
                                    //     println!("Argument = {:?}", buff);
                                    //     buff
                                    // } else {
                                    //     panic!("argument not found")
                                    // };
                                    let argument = &values[1];
                                    match command.as_slice(&buf) {
                                        b"ECHO" => {
                                            println!("ECHO was called");
                                            let output = encode(argument, &buf);
                                            println!("Output: {:?}", output);
                                            stream.write_all(output.as_bytes()).unwrap();
                                        }
                                        b"PING" => stream.write_all(b"+PONG\r\n").unwrap(),
                                        _ => println!("Something else called"),
                                    }
                                }
                                Some((pos, RedisBufSplit::String(value))) => {
                                    println!("Position: {}, Values: {:?}", pos, value);
                                }
                                Some((pos, RedisBufSplit::Int(value))) => {
                                    println!("Position: {}, Values: {:?}", pos, value);
                                }
                                Some((pos, RedisBufSplit::Error(value))) => {
                                    println!("Position: {}, Values: {:?}", pos, value);
                                }
                                Some((pos, RedisBufSplit::NullArray)) => {
                                    println!("Position: {}  ", pos);
                                }
                                Some((pos, RedisBufSplit::NullBulkString)) => {
                                    println!("Position: {}  ", pos);
                                }
                                None => println!("None arm matched"),
                            },
                        }
                        // match redis_parse(&buf[..bytes_read].to_vec(), 0).unwrap() {
                        //     Some((pos, value)) => {
                        //         println!("value: {:?}", value);
                        //     }
                        //     None => println!("None"),
                        // }
                    }
                });
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
    }
}
fn encode(value: &RedisBufSplit, buf: &Vec<u8>) -> String {
    let converted_string = match value {
        RedisBufSplit::String(string_value) => {
            format!(
                "${}\r\n{}\r\n",
                string_value.len(),
                String::from_utf8(string_value.as_slice(buf).to_vec()).unwrap()
            )
        }
        _ => "something".to_string(),
    };
    converted_string
}

// pub enum RedisCommand {
//     PING,
//     ECHO { data: String },
// }
// pub fn parse_command_array(buf: &mut [u8]) -> RedisCommand {
//     let mut index = 0;
//     assert!(buf[index] == b'*');
//     index += 1; //1
//     let array_length = (buf[index] - b'0') as usize;
//     if array_length == 1 {
//         return RedisCommand::PING;
//     }
//     index += 1; //2
//     index += CR_LEN + LF_LEN; //6
//     let bulk_string_symbol = buf[index];
//     assert!(bulk_string_symbol == b'$');
//     index += 1; //7
//     let command_length = (buf[index] - b'0') as usize;
//     index += 1; //8
//     index += CR_LEN + LF_LEN; //12
//     let command_string = String::from_utf8(buf[index..(index + command_length)].to_vec()).unwrap();
//     index += command_length; //16 if ECHO is the command
//     index += CR_LEN + LF_LEN; //20
//     let bulk_string_symbol = buf[index];
//     assert!(bulk_string_symbol == b'$');
//     index += 1; //21
//     let command_argument_length = (buf[index] - b'0') as usize;
//     index += 1; //8
//     index += CR_LEN + LF_LEN; //12
//     let command_argument_string =
//         String::from_utf8(buf[index..(index + command_argument_length)].to_vec()).unwrap();
//     println!("{:?}", command_argument_string);
//     // assert!(command_argument_string == "hey");
//     if command_string == "ECHO" {
//         return RedisCommand::ECHO {
//             data: command_argument_string,
//         };
//     }
//     RedisCommand::PING
// }
// match stream {
//     Ok(mut stream) => {
//         println!("accepted new connection");
//         spawn(move || {
//             let mut buf = [0; 64];
//             loop {
//                 let bytes_read = stream.read(&mut buf).unwrap();
//                 println!("Number of Bytes: {} \n Data: {:?}", bytes_read, buf);
//                 parse_command_array(&mut buf[..bytes_read]);
//                 if bytes_read == 0 {
//                     return;
//                 }
//                 let echo_command = parse_command_array(&mut buf);
//                 match echo_command {
//                     RedisCommand::PING => stream.write_all(b"+PONG\r\n").unwrap(),
//                     RedisCommand::ECHO { data } => {
//                         let response_string = &data;
//                         let response_string_len = &data.len();
//                         let output =
//                             format!("${}\r\n{}\r\n", response_string_len, response_string);
//                         stream.write_all(output.as_bytes()).unwrap();
//                     }
//                 }
//             }
//         });
//     }
//     Err(e) => {
//         println!("error: {}", e);
//     }
// }

fn word(buf: &Vec<u8>, pos: usize) -> Option<(usize, BufSplit)> {
    if buf.len() <= pos {
        return None;
    }
    memchr(b'\r', &buf[pos..]).and_then(|end| {
        if end + 1 < buf.len() {
            Some((pos + end + 2, BufSplit(pos, pos + end)))
        } else {
            None
        }
    })
}

fn simple_string(buf: &Vec<u8>, pos: usize) -> RedisResult {
    match word(buf, pos) {
        Some((pos, word)) => Ok(Some((pos, RedisBufSplit::String(word)))),
        None => Ok(None),
    }
}

// fn simple_string(buf: &Vec<u8>, pos: usize) -> RedisResult {
//     Ok(word(buf, pos).map(|(pos, word)| (pos, RedisBufSplit::String(word))))
// }

fn error(buf: &Vec<u8>, pos: usize) -> RedisResult {
    match word(buf, pos) {
        Some((pos, word)) => Ok(Some((pos, RedisBufSplit::Error(word)))),
        None => Ok(None),
    }
}

fn int(buf: &Vec<u8>, pos: usize) -> Result<Option<(usize, i64)>, RESPError> {
    match word(buf, pos) {
        Some((pos, word)) => {
            let s = str::from_utf8(word.as_slice(buf)).map_err(|_| RESPError::IntParseFailure)?;
            let i = s.parse().map_err(|_| RESPError::IntParseFailure)?;
            Ok(Some((pos, i)))
        }
        None => Ok(None),
    }
}

fn resp_int(buf: &Vec<u8>, pos: usize) -> RedisResult {
    Ok(int(buf, pos)?.map(|(pos, int)| (pos, RedisBufSplit::Int(int))))
}

fn bulk_string(buf: &Vec<u8>, pos: usize) -> RedisResult {
    match int(buf, pos)? {
        Some((pos, -1)) => Ok(Some((pos, RedisBufSplit::NullBulkString))),
        Some((pos, size)) if size >= 0 => {
            let total_size = pos + size as usize;
            if buf.len() < total_size + 2 {
                Ok(None)
            } else {
                let bulk_string = RedisBufSplit::String(BufSplit(pos, total_size));
                Ok(Some((total_size + 2, bulk_string)))
            }
        }
        Some((pos, bad_size)) => Err(RESPError::BadBulkStringSize(bad_size)),
        None => Ok(None),
    }
}

fn redis_parse(buf: &Vec<u8>, pos: usize) -> RedisResult {
    if buf.is_empty() {
        println!("Buffer is empty");
        return Ok(None);
    }
    match buf[pos] {
        b'+' => simple_string(buf, pos + 1),
        b'-' => error(buf, pos + 1),
        b'$' => bulk_string(buf, pos + 1),
        b':' => resp_int(buf, pos + 1),
        b'*' => array(buf, pos + 1),
        _ => Err(RESPError::UnknownStartingByte),
    }
}

fn array(buf: &Vec<u8>, pos: usize) -> RedisResult {
    match int(buf, pos)? {
        None => Ok(None),
        Some((pos, -1)) => Ok(Some((pos, RedisBufSplit::NullArray))),
        Some((pos, num_elements)) if num_elements >= 0 => {
            let mut values = Vec::with_capacity(num_elements as usize);
            let mut curr_pos = pos;
            for _ in 0..num_elements {
                match redis_parse(buf, curr_pos)? {
                    Some((new_pos, value)) => {
                        curr_pos = new_pos;
                        values.push(value);
                    }
                    None => return Ok(None),
                }
            }
            Ok(Some((curr_pos, RedisBufSplit::Array(values))))
        }
        Some((_pos, bad_num_elements)) => Err(RESPError::BadArraySize(bad_num_elements)),
    }
}
