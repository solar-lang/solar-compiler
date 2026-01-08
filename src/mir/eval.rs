use crate::mir::{CustomInstructionCode, Instruction};
use crate::value::Value;
fn string_concat(args: &[super::StaticExpression]) -> String {
    let mut buffer = String::new();
    for arg in args {
        let value = evaluate(&arg.instr);
        let str = value.to_string();
        buffer.push_str(&str);
    }
    buffer
}
