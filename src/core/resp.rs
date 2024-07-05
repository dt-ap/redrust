use anyhow::anyhow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Int64(i64),
    String(String),
    Vector(Vec<Value>),
    Empty,
}

type PositionAndValue = (usize, Value);

type Result = anyhow::Result<PositionAndValue>;

fn read_length(data: &[u8]) -> (usize, i32) {
    let mut length = 0_i32;

    for (pos, _) in data.into_iter().enumerate() {
        let b = data[pos];
        if !(b >= b'0' && b <= b'9') {
            return (pos + 2, length);
        }

        length = length * 10 + (b - b'0') as i32;
    }

    return (0, 0);
}

fn read_simple_string(data: &[u8]) -> Result {
    let mut pos = 1_usize;
    while data[pos] != b'\r' {
        pos += 1;
    }

    let simp_str = String::from_utf8(data[1..pos].to_vec())?;

    return Ok((pos + 2, Value::String(simp_str)));
}

fn read_error(data: &[u8]) -> Result {
    return read_simple_string(data);
}

fn read_i64(data: &[u8]) -> Result {
    let mut pos = 1_usize;
    let mut value = 0_i64;

    while data[pos] != b'\r' {
        value = value * 10 + (data[pos] - b'0') as i64;
        pos += 1;
    }

    return Ok((pos + 2, Value::Int64(value)));
}

fn read_bulk_string(data: &[u8]) -> Result {
    let mut pos = 1_usize;
    let (delta, len) = read_length(&data[pos..]);
    pos += delta;

    let len = len as usize;
    let bulk_str = String::from_utf8(data[pos..(pos + len)].to_vec())?;

    return Ok((pos + len + 2, Value::String(bulk_str)));
}

fn read_array(data: &[u8]) -> Result {
    let mut pos = 1_usize;
    let (delta, len) = read_length(&data[pos..]);
    pos += delta;

    let mut elems: Vec<Value> = Vec::with_capacity(len as usize);

    for _ in 0..len {
        let (delta, value) = decode_one(&data[pos..])?;

        elems.push(value);
        pos += delta;
    }

    return Ok((pos, Value::Vector(elems)));
}

pub fn decode_one(data: &[u8]) -> Result {
    if data.len() == 0 {
        return Err(anyhow!("No data"));
    }

    return match data[0] {
        b'+' => read_simple_string(data),
        b'-' => read_error(data),
        b':' => read_i64(data),
        b'$' => read_bulk_string(data),
        b'*' => read_array(data),
        _ => Ok((0, Value::Empty)),
    };
}

pub fn decode(data: &[u8]) -> anyhow::Result<Value> {
    if data.len() == 0 {
        return Err(anyhow!("No data"));
    }

    let (_, value) = decode_one(data)?;
    return Ok(value);
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_simple_string_decode() {
        let cases: HashMap<String, Value> =
            HashMap::from([("+OK\r\n".to_owned(), Value::String("OK".to_owned()))]);

        for (k, v) in cases.into_iter() {
            let data = decode(k.as_bytes()).unwrap();
            assert_eq!(data, v);
        }
    }

    #[test]
    fn test_error() {
        let cases: HashMap<String, Value> = HashMap::from([(
            "-Error message\r\n".to_owned(),
            Value::String("Error message".to_owned()),
        )]);

        for (k, v) in cases.into_iter() {
            let data = decode(k.as_bytes()).unwrap();
            assert_eq!(data, v);
        }
    }

    #[test]
    fn test_int_64() {
        let cases: HashMap<String, Value> = HashMap::from([
            (":0\r\n".to_owned(), Value::Int64(0)),
            (":1000\r\n".to_owned(), Value::Int64(1000)),
        ]);

        for (k, v) in cases.into_iter() {
            let data = decode(k.as_bytes()).unwrap();
            assert_eq!(data, v);
        }
    }

    #[test]
    fn test_array_decode() {
        let cases: HashMap<String, Value> = HashMap::from([
            ("*0\r\n".to_owned(), Value::Vector([].to_vec())),
            (
                "*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n".to_owned(),
                Value::Vector(vec![
                    Value::String("hello".to_owned()),
                    Value::String("world".to_owned()),
                ]),
            ),
            (
                "*3\r\n:1\r\n:2\r\n:3\r\n".to_owned(),
                Value::Vector(vec![
                    Value::Int64(1),
                    Value::Int64(2),
                    Value::Int64(3),
                ]),
            ),
            (
                "*2\r\n*3\r\n:1\r\n:2\r\n:3\r\n*2\r\n+Hello\r\n-World\r\n".to_owned(),
                Value::Vector(vec![
                    Value::Vector(vec![
                        Value::Int64(1),
                        Value::Int64(2),
                        Value::Int64(3),
                    ]),
                    Value::Vector(vec![
                        Value::String("Hello".to_owned()),
                        Value::String("World".to_owned()),
                    ]),
                ]),
            ),
        ]);

        for (k, v) in cases.into_iter() {
            let data = decode(k.as_bytes()).unwrap();
            assert_eq!(data, v);
        }
    }
}
