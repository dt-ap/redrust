#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Int64(i64),
    Int32(i32),
    String(String),
    Vector(Vec<Value>),
    Empty,
}
