use memchr::memchr;

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
pub struct BufSplit(usize, usize);

impl BufSplit {
    pub fn as_slice<'a>(&self, buf: &'a Vec<u8>) -> &'a [u8] {
        &buf[self.0..self.1]
    }
    fn len(&self) -> usize {
        self.1 - self.0 + 1
    }
}

#[derive(Debug)]
pub enum RedisBufSplit {
    String(BufSplit),
    Error(BufSplit),
    Int(i64),
    Array(Vec<RedisBufSplit>),
    NullArray,
    NullBulkString,
}
pub fn redis_buf_split_to_string(value: &RedisBufSplit, buf: &Vec<u8>) -> String {
    let string = match value {
        RedisBufSplit::String(string_value) => {
            String::from_utf8(string_value.as_slice(buf).to_vec()).unwrap()
        }
        _ => panic!("Not String"),
    };
    string
}
pub fn buf_split_to_string(value: &BufSplit, buf: &Vec<u8>) -> String {
    let string = String::from_utf8(value.as_slice(buf).to_vec()).unwrap();
    string
}
pub fn encode(value: &RedisBufSplit, buf: &Vec<u8>) -> String {
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
// pub fn encode_string(value: &Vec<u8>) -> String {
//     let converted_string = match value {
//         RedisBufSplit::String(string_value) => {
//             format!(
//                 "${}\r\n{}\r\n",
//                 string_value.len(),
//                 String::from_utf8(string_value.as_slice(buf).to_vec()).unwrap()
//             )
//         }
//         _ => "something".to_string(),
//     };
//     converted_string
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

pub fn redis_parse(buf: &Vec<u8>, pos: usize) -> RedisResult {
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
