use parser::{ParseResult, ResizableLimits};
use parser::memory_section::MemoryType;
use parser::table_section::TableEntry;
use parser::global_section::{GlobalType, GlobalVariable};

pub trait Validate {
    fn is_valid(&self) -> bool;
}

macro_rules! validate {
    ($ expr : expr) => (
        if ! $expr.is_valid() {
            return false;
        } else {
            println!("{:?} is valid!", $expr);
        }
    )
}

// generic validators

impl<T> Validate for Vec<T>
where
    T: Validate,
{
    fn is_valid(&self) -> bool {
        for e in self {
            if !e.is_valid() {
                return false;
            }
        }
        return true;
    }
}

impl<T> Validate for Option<T>
where
    T: Validate,
{
    fn is_valid(&self) -> bool {
        match self.as_ref() {
            Some(o) => o.is_valid(),
            None => true,
        }
    }
}

// concrete validator implementations

impl Validate for ResizableLimits {
    fn is_valid(&self) -> bool {
        match self.maximum {
            Some(max) => self.initial <= max,
            None => true,
        }
    }
}

impl Validate for MemoryType {
    fn is_valid(&self) -> bool {
        self.limits.is_valid()
    }
}

impl Validate for TableEntry {
    fn is_valid(&self) -> bool {
        self.limits.is_valid()
    }
}

impl Validate for GlobalType {
    fn is_valid(&self) -> bool {
        true
    }
}

impl Validate for GlobalVariable {
    fn is_valid(&self) -> bool {
        self.typ.is_valid()
    }
}

// validator struct

pub struct Validator {
    parse_result: ParseResult,
}

impl Validator {
    pub fn new(res: ParseResult) -> Validator {
        Validator { parse_result: res }
    }
    pub fn validate(&self) -> bool {
        validate!(self.parse_result.memory_types);
        validate!(self.parse_result.table_entries);
        validate!(self.parse_result.global_variables);
        return true;
    }
}
