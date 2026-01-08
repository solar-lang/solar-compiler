mod custom;
mod eval;
use crate::id::{FunctionId, TypeId};

pub use custom::CustomInstructionCode;
/// Expression with type-information
#[derive(Debug)]
pub struct StaticExpression {
    pub instr: Box<Instruction>,
    // NOTE maybe make this into an array of typeIDs later
    pub ty: TypeId,
}

#[derive(Debug)]
pub enum Instruction {
    /// Interface for compiler buildins etc.
    /// May be removed in the future.
    /// May be expanded for traps etc. Who knows.
    Custom {
        code: CustomInstructionCode,
        args: Vec<StaticExpression>,
    },
    Const(crate::value::Value),
    // Void,
    FunctionCall {
        func: FunctionId,
        args: Vec<StaticExpression>,
    },
    /// Get local Variable at address
    GetLocalVar(usize),
    /// Define a new let binding, that can be referenced later
    NewLocalVar {
        // name: String,
        // within the current scope the index is unique
        var_index: u16,
        // The value the variable will hold.
        var_value: StaticExpression,
        /// The expressions coming after the let binding,
        /// where the variable is in scope.
        body: StaticExpression,
    },
    IfExpr {
        /// Must be of typeId == Boolean
        condition: StaticExpression,
        case_true: StaticExpression,
        case_false: StaticExpression,
    }, // we have differnt kinds of for loops and ifs.
       // Expressions and Statements
       // (loops) Expressions return an Array of static types, that must be the same
}

impl Instruction {
    pub fn expr(self, ty: TypeId) -> StaticExpression {
        StaticExpression {
            instr: Box::new(self),
            ty,
        }
    }
}
