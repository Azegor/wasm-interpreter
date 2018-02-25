use parser::{Parser, Type};

#[derive(Debug)]
struct FuncType(Type, Vec<Type>, Option<Type>);

impl Parser {
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
    pub fn parse_type_section(&mut self, payload_len: u32) {
        println!("  # Parsing type section");
        let init_offset = self.get_current_offset();
        let types = self.read_vu32_times(Parser::read_func_type);
        assert!(self.get_read_len(init_offset) == payload_len as u64);
        println!("{:?}", types);
        println!("  + Parsing type section done");
        // return types
    }
}
