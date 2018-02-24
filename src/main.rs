extern crate byteorder;

use std::fs::File;
use std::io::Read;
use std::io::{Seek, SeekFrom};
use std::io::BufReader;
use std::string::String;
use byteorder::{LittleEndian, ReadBytesExt};

static MAGIC_NUM: u32 = 0x6d736100;
static SUPPORTED_VERSION: u32 = 0x1;
//static endOpcode: i32 = 0x0b;

pub fn to_hex_string(bytes: Vec<u8>) -> String {
    let strs: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
    strs.join(" ")
}

#[derive(Debug)]
struct FnId(u32);

#[derive(Debug)]
enum NameType {
    Unknown = -1,
    Module = 0,
    Function = 1,
    Local = 2,
}

impl NameType {
    fn from_int(int: u8) -> NameType {
        match int {
            0 => NameType::Module,
            1 => NameType::Function,
            2 => NameType::Local,
            _ => NameType::Unknown,
        }
    }
}

#[derive(Debug)]
struct ResizableLimits(bool, u32, Option<u32>);

#[derive(Debug)]
enum ExternalKind {
    Func(u32),
    Table(u8, ResizableLimits),
    Memory(ResizableLimits),
    Global(Type, bool),
}

#[derive(Debug)]
struct ImportEntry(String, String, ExternalKind);

#[derive(Debug)]
enum Type {
    I32 = 0x7f,
    I64 = 0x7e,
    F32 = 0x7d,
    F64 = 0x7c,
    anyfunc = 0x70,
    func = 0x60,
    empty_block = 0x40,
}

impl Type {
    fn from_int(int: u8) -> Type {
        match int {
            0x7f => Type::I32,
            0x7e => Type::I64,
            0x7d => Type::F32,
            0x7c => Type::F64,
            0x70 => Type::anyfunc,
            0x60 => Type::func,
            0x40 => Type::empty_block,
            _ => panic!("unknown Type value!"),
        }
    }

    fn value_type(int: u8) -> Type {
        match int {
            0x7f => Type::I32,
            0x7e => Type::I64,
            0x7d => Type::F32,
            0x7c => Type::F64,
            _ => panic!("unknown ValueType value!"),
        }
    }

    fn block_type(int: u8) -> Type {
        match int {
            0x7f => Type::I32,
            0x7e => Type::I64,
            0x7d => Type::F32,
            0x7c => Type::F64,
            0x40 => Type::empty_block,
            _ => panic!("unknown BlockType value!"),
        }
    }

    fn elem_type(int: u8) -> Type {
        match int {
            0x70 => Type::anyfunc,
            _ => panic!("unknown ElemType value!"),
        }
    }

    fn func_type(int: u8) -> Type {
        match int {
            0x60 => Type::func,
            _ => panic!("unknown FuncType value!"),
        }
    }
}

#[derive(Debug)]
struct Naming {
    index: u32,
    name: String,
}

impl Naming {
    fn new(idx: u32, n: String) -> Naming {
        Naming {
            index: idx,
            name: n,
        }
    }
}

/*
impl std::fmt::Debug for Naming {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}: '{}')", self.index, self.name)
    }
}
*/

#[derive(Debug)]
struct FuncType(Type, Vec<Type>, Option<Type>);

pub struct Parser {
    file: BufReader<File>,
}

impl Parser {
    fn new() -> Parser {
        Parser {
            file: BufReader::new(File::open("examples/wasm_test.wasm").unwrap()),
        }
    }

    fn get_current_offset(&mut self) -> u64 {
        self.file.seek(SeekFrom::Current(0)).unwrap()
    }

    fn get_read_len(&mut self, old: u64) -> u64 {
        self.get_current_offset() - old
    }

    fn read_byte(&mut self) -> u8 {
        self.file.read_u8().unwrap()
    }

    fn read_bytes(&mut self, len: u32) -> Vec<u8> {
        let mut name_bytes = vec![0u8; len as usize];
        self.file.read_exact(&mut name_bytes).unwrap();
        return name_bytes;
    }

    fn read_utf8(&mut self, len: u32) -> String {
        let name_bytes = self.read_bytes(len);
        String::from_utf8(name_bytes).unwrap()
    }

    fn read_uint32(&mut self) -> u32 {
        self.file.read_u32::<LittleEndian>().unwrap()
    }

    fn read_int32(&mut self) -> i32 {
        self.file.read_i32::<LittleEndian>().unwrap()
    }

    fn read_uint64(&mut self) -> u64 {
        self.file.read_u64::<LittleEndian>().unwrap()
    }

    fn read_int64(&mut self) -> i64 {
        self.file.read_i64::<LittleEndian>().unwrap()
    }

    fn read_varuint_len(&mut self, len: i32) -> (u64, u64) {
        let mut res: u64 = 0;
        let mut shift = 0;
        let mut read_bytes: u64 = 0;
        loop {
            read_bytes += 1;
            let byte = self.read_byte();
            res |= (byte as u64 & 0x7f) << shift;
            if (byte & 0x80) == 0 {
                break;
            }
            shift += 7;
        }
        assert!(read_bytes <= (len as f32 / 7.0).ceil() as u64);
        return (res, read_bytes);
    }

    fn read_varuint(&mut self, len: i32) -> u64 {
        self.read_varuint_len(len).0
    }

    fn read_varuint1(&mut self) -> bool {
        self.read_varuint(7) != 0
    }

    fn read_varuint7(&mut self) -> u8 {
        self.read_varuint(7) as u8
    }

    fn read_varuint32(&mut self) -> u32 {
        self.read_varuint(32) as u32
    }

    fn read_varuint64(&mut self) -> u64 {
        self.read_varuint(64)
    }

    fn read_varint_len(&mut self, len: i32) -> (i64, u64) {
        let mut res: i64 = 0;
        let mut shift = 0;
        let mut read_bytes: u64 = 0;
        let byte: u8 = 0;
        loop {
            read_bytes += 1;
            let byte = self.read_byte();
            res |= (0x7f & byte as i64) << shift;
            shift += 7;
            if (byte & 0x80) == 0 {
                break;
            }
        }
        if shift < len && (byte & 0x40) != 0 {
            res |= !0i64 << shift;
        }
        assert!(read_bytes <= (len as f32 / 7.0).ceil() as u64);
        return (res, read_bytes);
    }

    fn read_varint(&mut self, len: i32) -> i64 {
        self.read_varint_len(len).0
    }

    // ----------

    fn read_name_map(&mut self) -> Vec<Naming> {
        let count = self.read_varuint32();
        let mut names = Vec::<Naming>::new();
        for _ in 0..count {
            let index = self.read_varuint32() as u32;
            let name_len = self.read_varuint32();
            let name_str = self.read_utf8(name_len);
            let naming = Naming::new(index, name_str);
            names.push(naming)
        }
        return names;
    }

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

    fn read_value_type(&mut self) -> Type {
        let ptype = self.read_varuint7();
        Type::value_type(ptype)
    }

    fn read_resizable_limits(&mut self) -> ResizableLimits {
        let limits_flag = self.read_varuint1();
        let limits_initial = self.read_varuint32();
        let limits_maximum = if limits_flag {
            Some(self.read_varuint32())
        } else {
            None
        };
        ResizableLimits(limits_flag, limits_initial, limits_maximum)
    }

    fn read_func_type(&mut self) -> FuncType {
        let form = Type::func_type(self.read_varuint7());
        let param_count = self.read_varuint32();
        let mut param_types = Vec::<Type>::new();
        for _ in 0..param_count {
            let param_type = self.read_value_type();
            param_types.push(param_type);
        }
        let return_count = self.read_varuint1();
        let return_type = if return_count {
            Some(self.read_value_type())
        } else {
            None
        };
        FuncType(form, param_types, return_type)
    }

    // ----------

    pub fn parse(&mut self) {
        self.parse_preamble();

        let file_len = self.file.get_ref().metadata().unwrap().len();
        while self.get_current_offset() < file_len {
            self.parse_section();
        }
    }

    fn parse_preamble(&mut self) {
        print!("Parsing WASM header ... ");

        let magic = self.read_uint32();
        if magic != MAGIC_NUM {
            panic!("Not a wasm file!");
        }
        let version = self.read_uint32();
        if version != SUPPORTED_VERSION {
            panic!("Unsupported Version!");
        }
        println!("done");
    }

    fn parse_section(&mut self) {
        print!(" ## Parsing section ...");
        let sec_id = self.read_varuint7();
        let payload_len = self.read_varuint32();
        let mut name_offset: u32 = 0;
        let mut name: String = String::new();
        if sec_id == 0 {
            let (name_len, name_len_field_size) = self.read_varuint_len(32);
            name_offset = (name_len_field_size + name_len) as u32;
            name = self.read_utf8(name_len as u32);
            println!("[name = '{}']", name)
        } else {
            println!("[id = {}]", sec_id);
        }
        let payload_data_len = payload_len - name_offset;

        match sec_id {
            0x0 => {
                if name == "name" {
                    self.parse_name_custom_section(payload_data_len);
                } else {
                    self.parse_custom_section(&name, payload_data_len); // some other custom section
                }
            }
            0x1 => self.parse_type_section(payload_data_len),
            0x2 => self.parse_import_section(payload_data_len),
            0x3 => self.parse_function_section(payload_data_len),
            0x4 => self.parse_section_todo(payload_data_len),
            0x5 => self.parse_section_todo(payload_data_len),
            0x6 => self.parse_section_todo(payload_data_len),
            0x7 => self.parse_section_todo(payload_data_len),
            0x8 => self.parse_section_todo(payload_data_len),
            0x9 => self.parse_section_todo(payload_data_len),
            0xA => self.parse_section_todo(payload_data_len),
            0xB => self.parse_section_todo(payload_data_len),
            _ => panic!("Unknown Section ID!"),
        }

        println!(" ++ Done parsing section");
    }

    fn parse_section_todo(&mut self, payload_len: u32) {
        println!("  # Parsing section (TODO)");
        self.read_bytes(payload_len);
        println!("  + Parsing section (TODO) done");
    }

    fn parse_name_custom_section(&mut self, payload_len: u32) {
        println!("  # Parsing name custom section");
        let init_offset = self.get_current_offset();

        let mut name_module_section: Option<String> = None;
        let mut name_function_section: Option<Vec<Naming>> = None;
        type LocalNaming = (u32, Vec<Naming>);
        let mut name_local_section: Option<Vec<LocalNaming>> = None;
        type OtherSubSec = (NameType, Vec<u8>);
        let mut name_subsections = Vec::<OtherSubSec>::new();

        while self.get_read_len(init_offset) < payload_len as u64 {
            let name_type = NameType::from_int(self.read_varuint7());
            let name_payload_len = self.read_varuint32();
            // enforce ordering and uniqueness of the sections with assertions
            match name_type {
                NameType::Module => {
                    assert!(
                        name_module_section.is_none() && name_function_section.is_none()
                            && name_local_section.is_none()
                    );
                    let name_len = self.read_varuint32();
                    let name_str = self.read_utf8(name_len);
                    name_module_section = Some(name_str);
                }
                NameType::Function => {
                    assert!(name_function_section.is_none() && name_local_section.is_none());
                    let name_map = self.read_name_map();
                    name_function_section = Some(name_map);
                }
                NameType::Local => {
                    assert!(name_local_section.is_none());
                    let count = self.read_varuint32();
                    let mut locals = Vec::<(u32, Vec<Naming>)>::new();
                    for _ in 0..count {
                        let index = self.read_varuint32();
                        let local_map = self.read_name_map();
                        let mut local = (index, local_map);
                        locals.push(local);
                    }
                    name_local_section = Some(locals);
                }
                _ => {
                    let name_payload_data = self.read_bytes(name_payload_len);
                    let name_payload = name_payload_data;
                    let subsection = (name_type, name_payload);
                    name_subsections.push(subsection);
                }
            }
        }
        assert!(self.get_read_len(init_offset) == payload_len as u64);
        let result = (
            name_module_section,
            name_function_section,
            name_local_section,
            name_subsections,
        );
        println!("{:?}", result);
        println!("  + Parsing name custom section done")
        //return result
    }

    fn parse_custom_section(&mut self, name: &str, payload_len: u32) {
        println!("  # Parsing custom section [name = '{}']", name);
        let payload = self.read_bytes(payload_len);
        println!(
            "Custom Section Data [len={}] = {}...",
            payload_len,
            to_hex_string(payload)
        );
        println!("  + Parsing custom section done")
        // return name, payload
    }

    fn parse_type_section(&mut self, payload_len: u32) {
        println!("  # Parsing type section");
        let init_offset = self.get_current_offset();
        let count = self.read_varuint32();
        let mut types = Vec::<FuncType>::new();
        for _ in 0..count {
            types.push(self.read_func_type());
        }
        assert!(self.get_read_len(init_offset) == payload_len as u64);
        println!("{:?}", types);
        println!("  + Parsing type section done");
        // return types
    }

    fn parse_import_section(&mut self, payload_len: u32) {
        println!("  # Parsing import section");
        let init_offset = self.get_current_offset();
        let count = self.read_varuint32();
        let mut entries = Vec::<ImportEntry>::new();
        for _ in 0..count {
            let module_len = self.read_varuint32();
            let module_str = self.read_utf8(module_len);
            let field_len = self.read_varuint32();
            let field_str = self.read_utf8(field_len);
            let typ = self.read_external_kind();
            let import_entry = ImportEntry(module_str, field_str, typ);
            entries.push(import_entry);
        }
        assert!(self.get_read_len(init_offset) == payload_len as u64);
        println!("{:?}", entries);
        println!("  + Parsing import section done");
        //return entries;
    }

    fn parse_function_section(&mut self, payload_len: u32) {
        println!("  # Parsing function section");
        let init_offset = self.get_current_offset();
        let count = self.read_varuint32();
        let mut types = Vec::<FnId>::new();
        for _ in 0..count {
            let t = FnId(self.read_varuint32());
            types.push(t);
        }
        assert!(self.get_read_len(init_offset) == payload_len as u64);
        println!("{:?}", types);
        println!("  + Parsing function section done");
        // return types
    }
}

fn main() {
    println!("WASM PARSER\n===========");
    let mut parser = Parser::new();
    parser.parse();
    println!("===========\nDONE");
}
