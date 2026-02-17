use crate::util::IdPath;

/// Denotes either a symbol in local scope
/// or in a module
pub enum Symbol {
    LocalVar { addr: u16, ty: TypeId },
    Global(SymbolId),
}

/// Points to a unit of code, such as a method declaration or a type
pub type SymbolId = (IdModule, IdFile, IdItem);

/// Static SymbolId,
/// because symbols to generic functions can generate multiple offsprings
pub type SSID = (SymbolId, Vec<TypeId>);

pub type IdModule = IdPath;
pub type IdFile = u16;

#[derive(Debug, Hash, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum IdItem {
    // Variable
    GlobalVar(u16),

    /// Pointing to a function declared in the global scope
    Func(u16),

    /// Points to a Type declared in the global scope
    Type(u16),

    /// Describes a auto-derived Method in terms of referencing the type and specific field OR Enum and specific Variant
    /// Note, enum variants MAY be constant
    /// Structure:
    /// (Position of Item in File,  Position of Function in Item)
    Method(u16, u16),
}

pub type TypeId = usize;
pub type FunctionId = usize;
