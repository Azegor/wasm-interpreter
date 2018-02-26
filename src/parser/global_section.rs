use parser::{Parser, Type};
use parser::opcode::InitExpr;

#[derive(Debug)]
struct GlobalType {
    content_type: Type,
    mutability: bool,
}

#[derive(Debug)]
struct GlobalVariable {
    typ: GlobalType,
    init: InitExpr,
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
    pub fn parse_global_section(&mut self, payload_len: u32) {
        println!("  # Parsing global section");
        let init_offset = self.get_current_offset();
        let globals = self.read_vu32_times(Parser::read_global_variable);
        assert!(self.get_read_len(init_offset) == payload_len);
        println!("{:?}", globals);
        println!("  + Parsing global section done");
        //return globals
    }
}
