use std::rc::Rc;

pub type GcString = Rc<String>;

/// Represents a Dynamically Typed Value
#[derive(Debug, Clone)]
pub enum Value {
    Void,
    Bool(bool),
    Int(Int),
    String(GcString),
}

#[derive(Debug, Clone, Copy)]
pub enum Int {
    Int64(i64),
    Int32(i32),
    Int16(i16),
    Int8(i8),
    Uint64(u64),
    Uint32(u32),
    Uint16(u16),
    Uint8(u8),
}
