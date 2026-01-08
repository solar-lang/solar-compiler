mod context;
use crate::mir::{CustomInstructionCode, Instruction};
use crate::value::Value;

use std::sync::RwLock;

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
    pub types: RwLock<HotelMap<SSID, Type>>,

    pub functions: RwLock<FunctionStore>,
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
            types,
            functions,
        }
    }
}

