use std::ops::Deref;

use hotel::HotelMap;

use crate::{
    id::{TypeId, SSID},
    mir::StaticExpression,
};

#[derive(Debug)]
pub enum FunctionInfo {
    /// Used to reserve functions.
    /// Needed, because in cases like recursion we need to know
    /// that a function exists, before we have completly compiled it.
    Partial,
    Complete {
        // TODO also add the name of the argument, for error messages.
        arg_type_ids: Vec<TypeId>,
        body: StaticExpression,
    },
}

/// Container to hold information about compiled functions
#[derive(Default)]
pub struct FunctionStore {
    functions: HotelMap<SSID, FunctionInfo>,
}

impl FunctionStore {
    pub fn new() -> Self {
        FunctionStore::default()
    }

    /// Reserve a slot in the hotel map
    pub fn reserve(&mut self, key: SSID) -> usize {
        self.functions.insert(key, FunctionInfo::Partial)
    }

    pub fn update_complete_function(
        &mut self,
        index: usize,
        args: Vec<TypeId>,
        body: StaticExpression,
    ) {
        self.functions.set_by_index(
            index,
            FunctionInfo::Complete {
                arg_type_ids: args,
                body,
            },
        );
    }
}

impl Deref for FunctionStore {
    type Target = HotelMap<SSID, FunctionInfo>;

    fn deref(&self) -> &Self::Target {
        &self.functions
    }
}
