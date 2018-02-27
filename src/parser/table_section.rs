use parser::{Parser, ResizableLimits, Type};

#[derive(Debug)]
pub struct TableEntry {
    typ: Type,
    limits: ResizableLimits,
}

impl Parser {
    fn read_table_type(&mut self) -> TableEntry {
        let typ = Type::elem_type(self.read_varuint7());
        let limits = self.read_resizable_limits();
        TableEntry { typ, limits }
    }
    pub fn parse_table_section(&mut self, payload_len: u32) -> Vec<TableEntry> {
        println!("  # Parsing table section");
        let init_offset = self.get_current_offset();
        let entries = self.read_vu32_times(Parser::read_table_type);
        assert!(self.get_read_len(init_offset) == payload_len);
        return entries;
    }
}
