extern crate byteorder;

use std::fs::File;
use std::io::Read;
use std::io::BufReader;
use byteorder::{LittleEndian, ReadBytesExt};

static MAGIC_NUM: u32 = 0x6d736100;
static SUPPORTED_VERSION: u32 = 0x1;
//static endOpcode: i32 = 0x0b;

struct Parser {
    file: BufReader<File>,
}

impl Parser {
    fn new() -> Parser {
        Parser {
            file: BufReader::new(File::open("examples/empty.wasm").unwrap()),
        }
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

    fn read_varuint_len(&mut self, len: i32) -> Option<(u64, i32)> {
        let mut res: u64 = 0;
        let mut shift = 0;
        let mut read_bytes = 0;
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
        assert!(read_bytes <= (len as f32 / 7.0).ceil() as i32);
        return Some((res, read_bytes));
    }

    fn read_varuint(&mut self, len: i32) -> u64 {
        self.read_varuint_len(len).unwrap().0
    }

    fn read_varint_len(&mut self, len: i32) -> Option<(i64, i32)> {
        let mut res: i64 = 0;
        let mut shift = 0;
        let mut read_bytes = 0;
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
        assert!(read_bytes <= (len as f32 / 7.0).ceil() as i32);
        return Some((res, read_bytes));
    }

    fn read_varint(&mut self, len: i32) -> i64 {
        self.read_varint_len(len).unwrap().0
    }

    fn parse(&mut self) {
        self.parse_preamble();

        while self.parse_section() {}
    }

    fn parse_preamble(&mut self) {
        print!("Parsing WASM header ... ");

        //let mut rdr = Cursor::new(file);
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
        let sec_id = match self.read_varuint_len(7) {
            Some((val, _)) => val,
            None => return false,
        };
        print!(" ## Parsing section ...");
        let payload_len = self.read_varuint(32);
        let mut name_offset = 0;
        if sec_id == 0 {
            let (name_len, name_len_field_size) = self.read_varuint_len(32).unwrap();
            name_offset = (name_len_field_size as i64) + (name_len as i64);
            let mut name_bytes = vec![0u8; name_len as usize];
            self.file.read_exact(&mut name_bytes).unwrap();
            let name = std::str::from_utf8(&name_bytes).unwrap();
            println!("[name = '{}']", name)
        } else {
            println!("[id = {}]", sec_id);
        }
        let payload_data_len = payload_len - (name_offset as u64);
        let mut tmp = vec![0u8; payload_data_len as usize];
        self.file.read_exact(&mut tmp).unwrap();
        println!(" ++ Done parsing section");
        return true;
    }
}

fn main() {
    println!("WASM PARSER\n===========");
    let mut parser = Parser::new();
    parser.parse();
    println!("===========\nDONE");
}
