use parser::{Parser, ResizableLimits, Type};

#[derive(Debug)]
enum ExternalKind {
    Func(u32),
    Table(u8, ResizableLimits),
    Memory(ResizableLimits),
    Global(Type, bool),
}

#[derive(Debug)]
struct ImportEntry(String, String, ExternalKind);

impl Parser {
    fn read_external_kind(&mut self) -> ExternalKind {
        match self.read_byte() {
            0 => self.read_ext_func_type(),
            1 => self.read_ext_table_type(),
            2 => self.read_ext_memory_type(),
            3 => self.read_ext_global_type(),
            _ => panic!("Unknown ExternalKind value"),
        }
    }

    fn read_ext_func_type(&mut self) -> ExternalKind {
        ExternalKind::Func(self.read_varuint32())
    }

    fn read_ext_table_type(&mut self) -> ExternalKind {
        let elem_type = self.read_varuint7();
        let limits = self.read_resizable_limits();
        ExternalKind::Table(elem_type, limits)
    }

    fn read_ext_memory_type(&mut self) -> ExternalKind {
        ExternalKind::Memory(self.read_resizable_limits())
    }

    fn read_ext_global_type(&mut self) -> ExternalKind {
        let content_type = self.read_value_type();
        let mutability = self.read_varuint1();
        ExternalKind::Global(content_type, mutability)
    }

    fn read_import_entry(&mut self) -> ImportEntry {
        let module_len = self.read_varuint32();
        let module_str = self.read_utf8(module_len);
        let field_len = self.read_varuint32();
        let field_str = self.read_utf8(field_len);
        let typ = self.read_external_kind();
        ImportEntry(module_str, field_str, typ)
    }

    pub fn parse_import_section(&mut self, payload_len: u32) {
        println!("  # Parsing import section");
        let init_offset = self.get_current_offset();
        let entries = self.read_vu32_times(Parser::read_import_entry);
        assert!(self.get_read_len(init_offset) == payload_len as u64);
        println!("{:?}", entries);
        println!("  + Parsing import section done");
        //return entries;
    }
}
