use parser::{FnId, Parser};
use parser::opcode::InitExpr;

#[derive(Debug)]
struct ElemSegment {
    index: u32,
    offset: InitExpr,
    elems: Vec<FnId>,
}

impl Parser {
    fn read_element(&mut self) -> ElemSegment {
        let index = self.read_varuint32();
        let offset = self.read_init_expr();
        let elems = self.read_vu32_times(Parser::read_fn_id);
        ElemSegment {
            index,
            offset,
            elems,
        }
    }

    pub fn parse_element_section(&mut self, payload_len: u32) {
        println!("  # Parsing element section");
        let init_offset = self.get_current_offset();
        let entries = self.read_vu32_times(Parser::read_element);
        assert!(self.get_read_len(init_offset) == payload_len);
        println!("{:?}", entries);
        println!("  + Parsing element section done");
        // return entries;
    }
}
