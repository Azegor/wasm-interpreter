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
enum NameType {
    Module = 0,
    Function = 1,
    Local = 2,
    Unknown = -1,
}

impl NameType {
    fn from_int(int: u64) -> NameType {
        match int {
            0 => NameType::Module,
            1 => NameType::Function,
            2 => NameType::Local,
            _ => NameType::Unknown,
        }
    }
}

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

impl std::fmt::Debug for Naming {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}: '{}')", self.index, self.name)
    }
}

struct Parser {
    file: BufReader<File>,
}

impl Parser {
    fn new() -> Parser {
        Parser {
            file: BufReader::new(File::open("examples/xor.wasm").unwrap()),
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

    // ----------

    fn parse(&mut self) {
        self.parse_preamble();

        let metadata = self.file.get_ref().metadata().unwrap();
        let file_len = metadata.len();
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
    }

    fn parse_section_todo(&mut self, payload_len: u32) {
        println!("  # Parsing section (TODO)");
        self.read_bytes(payload_len);
        println!("  + Parsing section (TODO) done");
    }

    fn parse_name_custom_section(&mut self, payload_len: u32) {
        println!("  # Parsing name custom section");
        let init_offset = self.get_current_offset();
        //name_module_section = None
        //name_function_section = None
        //name_local_section = None
        //name_subsections = []
        while self.get_read_len(init_offset) < payload_len as u64 {
            let name_type = NameType::from_int(self.read_varuint(7));
            let name_payload_len = self.read_varuint32();
            // enforce ordering and uniqueness of the sections with assertions
            match name_type {
                NameType::Module => {
                    //assert!(name_module_section is None and name_function_section is None and name_local_section is None);
                    let name_len = self.read_varuint32();
                    let name_str = self.read_utf8(name_len);
                    println!("{}", name_str);
                    // name_module_section = (name_str,)
                }
                NameType::Function => {
                    //assert name_function_section is None and name_local_section is None
                    let name_map = self.read_name_map();
                    println!("{:?}", name_map);
                    //name_function_section = name_map
                }
                NameType::Local => {
                    //assert name_local_section is None
                    let count = self.read_varuint32();
                    // let funcs = [];
                    for _ in 0..count {
                        let index = self.read_varuint32();
                        let local_map = self.read_name_map();
                        println!("{}: {:?}", index, local_map);
                        //func = (index, local_map);
                        // funcs.push(func);
                    }
                    // name_local_section = funcs
                }
                _ => {
                    let name_payload_data = self.read_bytes(name_payload_len);
                    let name_payload = name_payload_data;
                    let subsection = (name_type, name_payload);
                    println!("{:?}: '{:?}'", subsection.0, subsection.1);
                    // name_subsections.append(subsection)
                }
            }
        }
        assert!(self.get_read_len(init_offset) == payload_len as u64);
        //result = (name_module_section, name_function_section, name_local_section, name_subsections)
        //println!(result)
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

    fn parse_function_section(&mut self, payload_len: u32) {
        println!("  # Parsing function section");
        let init_offset = self.get_current_offset();
        let count = self.read_varuint32();
        let mut types = Vec::<u32>::new();
        for _ in 0..count {
            let t = self.read_varuint32() as u32;
            types.push(t);
        }
        assert!(self.get_read_len(init_offset) == payload_len as u64);
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
