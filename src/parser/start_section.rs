use parser::{FnId, Parser};

impl Parser {
    pub fn parse_start_section(&mut self, payload_len: u32) {
        println!("  # Parsing start section");
        let (index, len) = self.read_varuint_len(32);
        let fn_id = FnId(index as u32);
        assert!(len == payload_len as u64);
        println!("{:?}", fn_id);
        println!("  + Parsing start section done");
        // return index;
    }
}
