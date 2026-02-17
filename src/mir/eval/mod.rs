use crate::compilation::FunctionInfo;
use crate::mir::Value;
use crate::mir::{CustomInstructionCode, Instruction};

use std::cell::RefCell;
use std::sync::Mutex;

use hotel::HotelMap;

use crate::{
    compilation::{CompilerContext, FunctionStore},
    id::SSID,
    types::{buildin::BuildinTypeId, Type},
};

pub struct EvaluationContext {
    /// IDs of buildin types like Int32 etc.
    pub buildin_types: BuildinTypeId,

    /// Contains static, concrete Type Information.
    pub types: HotelMap<SSID, Type>,

    pub functions: FunctionStore,

    stack: RefCell<Vec<Value>>,
}

impl<'a> From<CompilerContext<'a>> for EvaluationContext {
    fn from(cc: CompilerContext) -> EvaluationContext {
        let CompilerContext {
            buildin_types,
            types,
            functions,
            ..
        } = cc;

        EvaluationContext {
            buildin_types,
            types: types.into_inner().expect("locking types"),
            functions: functions.into_inner().expect("locking functions"),
            stack: RefCell::new(Vec::new()),
        }
    }
}

impl EvaluationContext {
    pub fn call(&mut self, func_id: usize, args: Vec<Value>) -> Value {
        let instruction = {
            let f = self
                .functions
                .get_by_index(func_id)
                .expect("receive valid function id");
            let FunctionInfo::Complete { args, body } = f else {
                panic!("Expected complete function, got Partial")
            };
            &body.instr
        };

        // Push values on stack.
        // NOTE this already assumes and further determines the calling convention.
        self.stack.borrow_mut().extend(args);

        // Call into the function
        self.eval_instruction(&instruction)
    }

    pub fn eval_instruction(&self, op: &Instruction) -> Value {
        match op {
            Instruction::Const(v) => v.clone(),
            Instruction::Custom { code, args } => match code {
                CustomInstructionCode::StrConcat => Value::String(self.string_concat(args)),
                CustomInstructionCode::Identity => {
                    // TODO don't evaluate functions
                    assert!(args.len() == 1, "expect only one argument to be passed");
                    self.eval_instruction(&args[0].instr)
                }
                CustomInstructionCode::Print => {
                    let text = self.string_concat(args);
                    print!("{text}");
                    Value::Void
                }
                CustomInstructionCode::Readline => {
                    if !args.is_empty() {
                        print!("{}", self.string_concat(args));
                    }
                    let mut buf = String::new();
                    std::io::stdin().read_line(&mut buf).expect("read line");
                    Value::String(buf)
                }
            },
            Instruction::FunctionCall { func, args } => todo!(),
            Instruction::GetLocalVar(_) => todo!(),
            Instruction::NewLocalVar {
                var_index,
                var_value,
                body,
            } => todo!(),
            Instruction::IfExpr {
                condition,
                case_true,
                case_false,
            } => todo!(),
        }
    }

    fn string_concat(&self, args: &[super::StaticExpression]) -> String {
        let mut buffer = String::new();
        for arg in args {
            let value = self.eval_instruction(&arg.instr);
            let str = value.to_string();
            buffer.push_str(&str);
        }
        buffer
    }
}
