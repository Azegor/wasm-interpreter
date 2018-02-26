use parser::{Parser, ResizableLimits, Type};

#[derive(Debug)]
enum ExternalKind {
    Func = 0,
    Table = 1,
    Memory = 2,
    Global = 3,
}

#[derive(Debug)]
enum ExternalKindType {
    Func(u32),
    Table(u8, ResizableLimits),
    Memory(ResizableLimits),
    Global(Type, bool),
}

#[derive(Debug)]
struct ImportEntry {
    module: String,
    field: String,
    kind: ExternalKind,
    typ: ExternalKindType,
}

#[derive(Debug)]
struct ExportEntry {
    field: String,
    kind: ExternalKind,
    index: u32,
}

impl Parser {
    fn read_external_kind(&mut self) -> ExternalKind {
        match self.read_byte() {
            0 => ExternalKind::Func,
            1 => ExternalKind::Table,
            2 => ExternalKind::Memory,
            3 => ExternalKind::Global,
            _ => panic!("Unknown ExternalKind value"),
        }
    }
    fn read_external_kind_and_type(&mut self) -> (ExternalKind, ExternalKindType) {
        let kind = self.read_external_kind();
        let typ = match kind {
            ExternalKind::Func => self.read_ext_func_type(),
            ExternalKind::Table => self.read_ext_table_type(),
            ExternalKind::Memory => self.read_ext_memory_type(),
            ExternalKind::Global => self.read_ext_global_type(),
        };
        return (kind, typ);
    }

    fn read_ext_func_type(&mut self) -> ExternalKindType {
        ExternalKindType::Func(self.read_varuint32())
    }

    fn read_ext_table_type(&mut self) -> ExternalKindType {
        let elem_type = self.read_varuint7();
        let limits = self.read_resizable_limits();
        ExternalKindType::Table(elem_type, limits)
    }

    fn read_ext_memory_type(&mut self) -> ExternalKindType {
        ExternalKindType::Memory(self.read_resizable_limits())
    }

    fn read_ext_global_type(&mut self) -> ExternalKindType {
        let content_type = self.read_value_type();
        let mutability = self.read_varuint1();
        ExternalKindType::Global(content_type, mutability)
    }

    fn read_import_entry(&mut self) -> ImportEntry {
        let module = self.read_utf8_str_vu32();
        let field = self.read_utf8_str_vu32();
        let (kind, typ) = self.read_external_kind_and_type();
        ImportEntry {
            module,
            field,
            kind,
            typ,
        }
    }

    pub fn parse_import_section(&mut self, payload_len: u32) {
        println!("  # Parsing import section");
        let init_offset = self.get_current_offset();
        let entries = self.read_vu32_times(Parser::read_import_entry);
        assert!(self.get_read_len(init_offset) == payload_len);
        println!("{:?}", entries);
        println!("  + Parsing import section done");
        //return entries;
    }
    fn read_export_entry(&mut self) -> ExportEntry {
        let field = self.read_utf8_str_vu32();
        let kind = self.read_external_kind();
        let index = self.read_varuint32();
        ExportEntry { field, kind, index }
    }

    pub fn parse_export_section(&mut self, payload_len: u32) {
        println!("  # Parsing export section");
        let init_offset = self.get_current_offset();
        let entries = self.read_vu32_times(Parser::read_export_entry);
        assert!(self.get_read_len(init_offset) == payload_len);
        println!("{:?}", entries);
        println!("  + Parsing export section done");
        //return entries;
    }
}
