use crate::mir::{CustomInstructionCode, Instruction};
use crate::value::Value;

fn evaluate(op: &Instruction) -> Value {
    match op {
        Instruction::Const(v) => v.clone(),
        Instruction::Custom { code, args } => match code {
            CustomInstructionCode::StrConcat => Value::String(string_concat(args)),
            CustomInstructionCode::Identity => {
                // TODO don't evaluate functions
                assert!(args.len() == 1, "expect only one argument to be passed");
                evaluate(&args[0].instr)
            }
            CustomInstructionCode::Print => {
                let text = string_concat(args);
                print!("{text}");
                Value::Void
            }
            CustomInstructionCode::Readline => {
                if !args.is_empty() {
                    print!("{}", string_concat(args));
                }
                let mut buf = String::new();
                std::io::stdin().read_line(buf).expect("read line");
                Value::String(buf)
            }
        },
    }
}

fn string_concat(args: &[super::StaticExpression]) -> String {
    let mut buffer = String::new();
    for arg in args {
        let value = evaluate(&arg.instr);
        let str = value.to_string();
        buffer.push_str(&str);
    }
    buffer
}
