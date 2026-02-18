use crate::compilation::FunctionInfo;
use crate::mir::{CustomInstructionCode, Instruction};
use crate::mir::{StaticExpression, Value};

use std::cell::RefCell;

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

    function_pointer: RefCell<Vec<usize>>,

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
            function_pointer: Vec::new().into(),
            stack: Vec::new().into(),
        }
    }
}

impl EvaluationContext {
    pub fn call(&self, func_id: usize, args: Vec<Value>) -> Value {
        // Load the function instructions
        let instruction = {
            let f = self
                .functions
                .get_by_index(func_id)
                .expect("receive valid function id");
            let FunctionInfo::Complete { body, .. } = f else {
                panic!("Expected complete function, got Partial")
            };
            &body.instr
        };

        // Set the function pointer to the current reference frame
        let stack_size = self.stack.borrow().len();
        self.function_pointer.borrow_mut().push(stack_size);

        // Push values on stack.
        // NOTE this already assumes and further determines the calling convention.
        self.stack.borrow_mut().extend(args);

        // Call into the function, save return value
        let ret = self.eval_instruction(&instruction);

        // reset function pointer
        // and also reset stack to size it had before
        self.function_pointer.borrow_mut().pop();
        self.stack.borrow_mut().truncate(stack_size);
        return ret;
    }

    pub fn eval_expression(&self, expr: &StaticExpression) -> Value {
        let i = &expr.instr;
        self.eval_instruction(i)
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
            Instruction::FunctionCall { func_id, args } => {
                let args: Vec<Value> = args
                    .iter()
                    .map(|s| self.eval_expression(&s))
                    .collect::<Vec<Value>>();

                self.call(*func_id, args)
            }
            Instruction::GetLocalVar(addr) => {
                let fp = self.fp();
                let v_addr = fp + addr;

                self.stack
                    .borrow()
                    .get(v_addr)
                    .expect("fp+addr to be valid index")
                    .clone()
            }
            Instruction::NewLocalVar {
                var_index,
                var_value,
                body,
            } => {
                // Note: this may well be deleted one day.
                // It's just a (pricy) runtime assertion,
                // that indices are computed correctly throughout compilation
                // calulate expected index of variable and compare with actual one.
                // expected index = sp - fp, so that fp+index= top of current stack
                assert!(
                    {
                        let expected_index = self.stack.borrow().len() - self.fp();
                        expected_index == *var_index as usize
                    },
                    "expect var index to be top of current function frame"
                );
                let value = self.eval_expression(var_value);
                // push value to stack
                self.stack.borrow_mut().push(value);
                let ret = self.eval_expression(body);
                // drop value from stack
                self.stack.borrow_mut().pop();

                ret
            }
            Instruction::IfExpr {
                condition,
                case_true,
                case_false,
            } => todo!(),
        }
    }

    /// Get the current function pointer
    fn fp(&self) -> usize {
        self.function_pointer.borrow().last().copied().unwrap_or(0)
    }

    fn string_concat(&self, args: &[super::StaticExpression]) -> String {
        let mut buffer = String::new();
        for arg in args {
            let value = self.eval_expression(&arg);
            let str = value.to_string();
            buffer.push_str(&str);
        }
        buffer
    }
}
