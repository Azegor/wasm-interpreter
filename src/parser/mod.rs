extern crate byteorder;

mod custom_section;
mod type_section;
mod import_section;
mod function_section;
mod table_section;
mod memory_section;
mod global_section;

mod opcode;

use std::fs::File;
use std::io::Read;
use std::io::{Seek, SeekFrom};
use std::io::BufReader;
use std::string::String;
use self::byteorder::{LittleEndian, ReadBytesExt};

static MAGIC_NUM: u32 = 0x6d736100;
static SUPPORTED_VERSION: u32 = 0x1;
//static endOpcode: i32 = 0x0b;

#[derive(Debug)]
struct ResizableLimits {
    flags: bool,
    initial: u32,
    maximum: Option<u32>,
}

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

pub struct Parser {
    file: BufReader<File>,
}

impl Parser {
    pub fn new() -> Parser {
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

    fn read_n_times<T>(&mut self, callback: fn(p: &mut Parser) -> T, n: u32) -> Vec<T> {
        let mut res = Vec::<T>::new();
        for _ in 0..n {
            res.push(callback(self));
        }
        return res;
    }

    fn read_vu32_times<T>(&mut self, callback: fn(p: &mut Parser) -> T) -> Vec<T> {
        let n = self.read_varuint32();
        self.read_n_times(callback, n)
    }

    // read functions used by multiple modules:

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
        ResizableLimits {
            flags: limits_flag,
            initial: limits_initial,
            maximum: limits_maximum,
        }
    }

    // ----------

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
            0x4 => self.parse_table_section(payload_data_len),
            0x5 => self.parse_memory_section(payload_data_len), // memory
            0x6 => self.parse_global_section(payload_data_len), // global
            0x7 => self.parse_section_todo(payload_data_len),   // export
            0x8 => self.parse_section_todo(payload_data_len),   // start
            0x9 => self.parse_section_todo(payload_data_len),   // element
            0xA => self.parse_section_todo(payload_data_len),   // code
            0xB => self.parse_section_todo(payload_data_len),   // data
            _ => panic!("Unknown Section ID!"),
        }

        println!(" ++ Done parsing section");
    }

    fn parse_section_todo(&mut self, payload_len: u32) {
        println!("  # Parsing section (TODO)");
        self.read_bytes(payload_len);
        println!("  + Parsing section (TODO) done");
    }

    pub fn parse(&mut self) {
        self.parse_preamble();

        let file_len = self.file.get_ref().metadata().unwrap().len();
        while self.get_current_offset() < file_len {
            self.parse_section();
        }
    }
}
