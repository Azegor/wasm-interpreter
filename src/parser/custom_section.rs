use parser::Parser;

fn to_hex_string(bytes: Vec<u8>) -> String {
    let strs: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
    strs.join(" ")
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

impl Parser {
    fn read_naming(&mut self) -> Naming {
        let index = self.read_varuint32() as u32;
        let name = self.read_utf8_str_vu32();
        Naming::new(index, name)
    }

    fn read_name_map(&mut self) -> Vec<Naming> {
        self.read_vu32_times(Parser::read_naming)
    }

    pub fn parse_name_custom_section(&mut self, payload_len: u32) {
        println!("  # Parsing name custom section");
        let init_offset = self.get_current_offset();

        let mut name_module_section: Option<String> = None;
        let mut name_function_section: Option<Vec<Naming>> = None;
        type LocalNaming = (u32, Vec<Naming>);
        let mut name_local_section: Option<Vec<LocalNaming>> = None;
        type OtherSubSec = (NameType, Vec<u8>);
        let mut name_subsections = Vec::<OtherSubSec>::new();

        while self.get_read_len(init_offset) < payload_len {
            let name_type = NameType::from_int(self.read_varuint7());
            let name_payload_len = self.read_varuint32();
            // enforce ordering and uniqueness of the sections with assertions
            match name_type {
                NameType::Module => {
                    assert!(
                        name_module_section.is_none() && name_function_section.is_none()
                            && name_local_section.is_none()
                    );
                    let name = self.read_utf8_str_vu32();
                    name_module_section = Some(name);
                }
                NameType::Function => {
                    assert!(name_function_section.is_none() && name_local_section.is_none());
                    let name_map = self.read_name_map();
                    name_function_section = Some(name_map);
                }
                NameType::Local => {
                    assert!(name_local_section.is_none());
                    fn read_local_entry(p: &mut Parser) -> (u32, Vec<Naming>) {
                        let index = p.read_varuint32();
                        let local_map = p.read_name_map();
                        (index, local_map)
                    }
                    let locals = self.read_vu32_times(read_local_entry);
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
        assert!(self.get_read_len(init_offset) == payload_len);
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

    pub fn parse_custom_section(&mut self, name: &str, payload_len: u32) {
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
}
