#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Int64(i64),
    String(String),
    Vector(Vec<Value>),
    Empty,
}
