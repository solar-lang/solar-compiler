use crate::mir::{CustomInstructionCode, Instruction};
use crate::value::Value;

fn evaluate(op: &Instruction) -> Value {
    match op {
        Instruction::Const(v) => v.clone(),

fn string_concat(args: &[super::StaticExpression]) -> String {
    let mut buffer = String::new();
    for arg in args {
        let value = evaluate(&arg.instr);
        let str = value.to_string();
        buffer.push_str(&str);
    }
    buffer
}
