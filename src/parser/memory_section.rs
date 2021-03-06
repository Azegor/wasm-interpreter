use parser::{Parser, ResizableLimits};

#[derive(Debug)]
pub struct MemoryType {
    pub limits: ResizableLimits,
}

impl Parser {
    fn read_memory_type(&mut self) -> MemoryType {
        MemoryType {
            limits: self.read_resizable_limits(),
        }
    }

    pub fn parse_memory_section(&mut self, payload_len: u32) -> Vec<MemoryType> {
        println!("  # Parsing memory section");
        let init_offset = self.get_current_offset();
        let entries = self.read_vu32_times(Parser::read_memory_type);
        assert_eq!(self.get_read_len(init_offset), payload_len);
        return entries;
    }
}
