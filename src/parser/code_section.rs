use parser::{Parser, Type};
use parser::opcode::{Op, Opcode};

#[derive(Debug)]
struct Local {
    count: u32,
    typ: Type,
}

#[derive(Debug)]
pub struct FnBody {
    locals: Vec<Local>,
    code: Vec<Op>,
}

impl Parser {
    fn read_local_entry(&mut self) -> Local {
        let count = self.read_varuint32();
        let typ = self.read_value_type();
        Local { count, typ }
    }

    fn read_fn_body(&mut self) -> FnBody {
        let body_size = self.read_varuint32();
        let body_head_offset = self.get_current_offset();
        let locals = self.read_vu32_times(Parser::read_local_entry);
        let body_head_size = self.get_read_len(body_head_offset);
        let codelen = body_size - body_head_size - 1;
        let mut code = Vec::<Op>::new();
        let code_offset = self.get_current_offset();
        while self.get_read_len(code_offset) < codelen {
            code.push(self.read_op());
        }
        let end = self.read_op();
        assert!(end.opcode == Opcode::end);
        FnBody { locals, code }
    }

    pub fn parse_code_section(&mut self, payload_len: u32) -> Vec<FnBody> {
        println!("  # Parsing code section");
        let init_offset = self.get_current_offset();
        let bodies = self.read_vu32_times(Parser::read_fn_body);
        assert_eq!(self.get_read_len(init_offset), payload_len);
        return bodies;
    }
}
