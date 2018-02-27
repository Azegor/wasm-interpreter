use parser::{Parser, Type};

#[derive(Debug)]
pub struct FuncType {
    form: Type,
    param_types: Vec<Type>,
    return_type: Option<Type>,
}

impl Parser {
    fn read_func_type(&mut self) -> FuncType {
        let form = Type::func_type(self.read_varuint7());
        let param_types = self.read_vu32_times(Parser::read_value_type);
        let return_type = if self.read_varuint1() {
            Some(self.read_value_type())
        } else {
            None
        };
        FuncType {
            form,
            param_types,
            return_type,
        }
    }
    pub fn parse_type_section(&mut self, payload_len: u32) -> Vec<FuncType> {
        println!("  # Parsing type section");
        let init_offset = self.get_current_offset();
        let types = self.read_vu32_times(Parser::read_func_type);
        assert_eq!(self.get_read_len(init_offset), payload_len);
        return types;
    }
}
