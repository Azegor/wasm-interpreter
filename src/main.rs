mod parser;
mod validator;

use parser::Parser;
use validator::Validator;

fn main() {
    println!("WASM PARSER\n===========");
    let mut parser = Parser::new();
    let res = parser.parse();
    println!("===========\nDONE");
    println!("===========COMPLETE PARSE RESULT:===========\n");
    println!("{:?}", res);
    println!("===========Validating===========");
    let validator = Validator::new(res);
    if !validator.validate() {
        println!("Invalid Module!");
        return;
    }
}
