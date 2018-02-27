use parser::{Parser, Type};
use parser::opcode::InitExpr;

#[derive(Debug)]
pub struct GlobalType {
    pub content_type: Type,
    pub mutability: bool,
}

#[derive(Debug)]
pub struct GlobalVariable {
    pub typ: GlobalType,
    pub init: InitExpr,
}

impl Parser {
    fn read_global_type(&mut self) -> GlobalType {
        let content_type = self.read_value_type();
        let mutability = self.read_varuint1();
        GlobalType {
            content_type,
            mutability,
        }
    }

    fn read_global_variable(&mut self) -> GlobalVariable {
        let typ = self.read_global_type();
        let init = self.read_init_expr();
        GlobalVariable { typ, init }
    }
    pub fn parse_global_section(&mut self, payload_len: u32) -> Vec<GlobalVariable> {
        println!("  # Parsing global section");
        let init_offset = self.get_current_offset();
        let globals = self.read_vu32_times(Parser::read_global_variable);
        assert_eq!(self.get_read_len(init_offset), payload_len);
        return globals;
    }
}
