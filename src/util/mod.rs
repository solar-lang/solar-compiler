mod scope;
pub use scope::*;
use solar_parser::{ast, ast::identifier::IdentifierPath};

use crate::types::buildin::BuildinTypeId;

/// Denotes an global identifier used to resolve
/// modules and symbols across libraries and versions of libraries.
pub type IdPath = Vec<String>;

/// The IdPath (that is, the common prefix)
/// of all symbols inside the current project
/// (e.g. the one with "./solar.yaml")
pub fn target_id() -> IdPath {
    vec!["self".to_string()]
}

/// Normalizing the path means appending modules we have imported to the path start.
/// At the moment this is not done and might be deleted entirely later on.
pub(crate) fn normalize_path(path: &IdentifierPath) -> Vec<String> {
    // TODO find path[0] in imports
    // and append this import to the start
    // NOTE: this is context dependent.
    // e.g. associated methods don't need to be in scope.

    path.value.iter().map(|i| i.value.to_string()).collect()
}

pub(crate) fn eval_int(
    int: &ast::expr::literal::Int,
    types: &BuildinTypeId,
) -> Result<(crate::mir::Int, u8), std::num::ParseIntError> {
    use crate::mir::Int;
    use ast::expr::literal::IntTypeSuffix as Ty;

    let type_hint = int.type_suffix.unwrap_or(Ty::Int);
    let radix = int.radix as u32;

    let i = match type_hint {
        Ty::Int => (
            Int::Int64(i64::from_str_radix(int.digits, radix)?),
            types.int,
        ),
        Ty::Int32 => (
            Int::Int32(i32::from_str_radix(int.digits, radix)?),
            types.int32,
        ),
        Ty::Int16 => (
            Int::Int16(i16::from_str_radix(int.digits, radix)?),
            types.int16,
        ),
        Ty::Int8 => (
            Int::Int8(i8::from_str_radix(int.digits, radix)?),
            types.int8,
        ),
        Ty::Uint => (
            Int::Uint64(u64::from_str_radix(int.digits, radix)?),
            types.uint,
        ),
        Ty::Uint32 => (
            Int::Uint32(u32::from_str_radix(int.digits, radix)?),
            types.uint32,
        ),
        Ty::Uint16 => (
            Int::Uint16(u16::from_str_radix(int.digits, radix)?),
            types.uint16,
        ),
        Ty::Uint8 => (
            Int::Uint8(u8::from_str_radix(int.digits, radix)?),
            types.uint8,
        ),
    };

    Ok(i)
}
