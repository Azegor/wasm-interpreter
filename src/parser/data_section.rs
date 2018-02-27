use parser::Parser;
use parser::opcode::InitExpr;

#[derive(Debug)]
pub struct DataEntry {
    index: u32,
    offset: InitExpr,
    data: Vec<u8>,
}

impl Parser {
    fn read_data_entry(&mut self) -> DataEntry {
        let index = self.read_varuint32();
        let offset = self.read_init_expr();
        let size = self.read_varuint32();
        let data = self.read_bytes(size);
        DataEntry {
            index,
            offset,
            data,
        }
    }

    pub fn parse_data_section(&mut self, payload_len: u32) -> Vec<DataEntry> {
        // custom name section needs to be parsed after the data section!
        //assert!(self.resData.name_section.is_none()); // TODO!
        println!("  # Parsing data section");
        let init_offset = self.get_current_offset();
        let entries = self.read_vu32_times(Parser::read_data_entry);
        assert_eq!(self.get_read_len(init_offset), payload_len);
        return entries;
    }
}
