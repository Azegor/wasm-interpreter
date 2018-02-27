mod parser;

use parser::Parser;

fn main() {
    println!("WASM PARSER\n===========");
    let mut parser = Parser::new();
    let res = parser.parse();
    println!("===========\nDONE");
    println!("===========COMPLETE PARSE RESULT:===========\n");
    println!("{:?}", res);
}
