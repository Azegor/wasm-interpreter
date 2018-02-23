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

struct Parser {
    file: BufReader<File>,
}

impl Parser {
    fn new() -> Parser {
        Parser {
            file: BufReader::new(File::open("examples/add.wasm").unwrap()),
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

    fn read_bytes(&mut self, len: u64) -> Vec<u8> {
        let mut name_bytes = vec![0u8; len as usize];
        self.file.read_exact(&mut name_bytes).unwrap();
        return name_bytes;
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

    fn read_varuint_impl(&mut self, len: i32) -> Option<(u64, u64)> {
        let mut res: u64 = 0;
        let mut shift = 0;
        let mut read_bytes: u64 = 0;
        loop {
            read_bytes += 1;
            let byte = match self.file.read_u8() {
                Ok(b) => b,
                Err(_) => return None,
            };
            res |= (byte as u64 & 0x7f) << shift;
            if (byte & 0x80) == 0 {
                break;
            }
            shift += 7;
        }
        assert!(read_bytes <= (len as f32 / 7.0).ceil() as u64);
        return Some((res, read_bytes));
    }

    fn read_varuint(&mut self, len: i32) -> u64 {
        self.read_varuint_impl(len).unwrap().0
    }

    fn read_varint_impl(&mut self, len: i32) -> Option<(i64, u64)> {
        let mut res: i64 = 0;
        let mut shift = 0;
        let mut read_bytes: u64 = 0;
        let byte: u8 = 0;
        loop {
            read_bytes += 1;
            let byte = match self.file.read_u8() {
                Ok(b) => b,
                Err(_) => return None,
            };
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
        return Some((res, read_bytes));
    }

    fn read_varint(&mut self, len: i32) -> i64 {
        self.read_varint_impl(len).unwrap().0
    }

    fn parse(&mut self) {
        self.parse_preamble();

        while self.parse_section() {}
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

    fn parse_section(&mut self) -> bool {
        // this is the only way to check for EOF that I know of!
        let sec_id = match self.read_varuint_impl(7) {
            Some((val, _)) => val,
            None => return false,
        };
        print!(" ## Parsing section ...");
        let payload_len = self.read_varuint(32);
        let mut name_offset: u64 = 0;
        let mut name: String = String::new();
        if sec_id == 0 {
            let (name_len, name_len_field_size) = self.read_varuint_impl(32).unwrap();
            name_offset = (name_len_field_size as u64) + name_len;
            let name_bytes = self.read_bytes(name_len);
            name = String::from_utf8(name_bytes).unwrap();
            println!("[name = '{}']", name)
        } else {
            println!("[id = {}]", sec_id);
        }
        let payload_data_len = payload_len - name_offset;

        println!("Payload length: {} bytes", payload_data_len);
        match sec_id {
            0x0 => {
                if name == "name" {
                    self.parse_name_custom_section(payload_data_len);
                } else {
                    self.parse_custom_section(&name, payload_data_len); // some other custom section
                }
            }
            0x1 => self.parse_section_todo(payload_data_len),
            0x2 => self.parse_section_todo(payload_data_len),
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
        return true;
    }

    fn parse_section_todo(&mut self, payload_len: u64) {
        println!("  # Parsing section (TODO)");
        self.read_bytes(payload_len);
        println!("  + Parsing section (TODO) done");
    }

    fn parse_name_custom_section(&mut self, payload_len: u64) {
        println!("  # Parsing name custom section");
        let payload = self.read_bytes(payload_len);
        println!("  + Parsing name custom section done");
    }

    fn parse_custom_section(&mut self, name: &str, payload_len: u64) {
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

    fn parse_function_section(&mut self, payload_len: u64) {
        println!("  # Parsing function section");
        let init_offset = self.get_current_offset();
        let count = self.read_varuint(32);
        let mut types = Vec::<u32>::new();
        for _ in 0..count {
            let t = self.read_varuint(32) as u32;
            types.push(t);
        }
        assert!(self.get_read_len(init_offset) == payload_len);
        //println!("{}", types);
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
