use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Int64(i64),
    Int32(i32),
    String(String),
    Vector(Vec<Value>),
    VectorString(Vec<String>),
    Empty,
}

impl Value {
    fn simple_to_str(&self) -> String {
        return match self {
            Value::String(s) => s.to_owned(),
            Value::Int64(i) => i.to_string(),
            Value::Int32(i) => i.to_string(),
            Value::VectorString(vs) => vs.join(", "),
            _ => "".to_string(),
        };
    }

    fn custom_to_string(&self) -> String {
        return match self {
            Value::Vector(vec) => vec
                .iter()
                .map(|v| v.simple_to_str())
                .collect::<Vec<String>>()
                .join(", "),
            _ => self.simple_to_str(),
        };
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.custom_to_string())?;
        return Ok(());
    }
}
