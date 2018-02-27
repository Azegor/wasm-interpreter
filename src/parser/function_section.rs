use parser::{FnId, Parser};

impl Parser {
    pub fn read_fn_id(&mut self) -> FnId {
        FnId(self.read_varuint32())
    }

    pub fn parse_function_section(&mut self, payload_len: u32) -> Vec<FnId> {
        println!("  # Parsing function section");
        let init_offset = self.get_current_offset();
        let ids = self.read_vu32_times(Parser::read_fn_id);
        assert!(self.get_read_len(init_offset) == payload_len);
        return ids;
    }
}
