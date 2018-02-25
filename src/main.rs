mod parser;

use parser::Parser;

fn main() {
    println!("WASM PARSER\n===========");
    let mut parser = Parser::new();
    parser.parse();
    println!("===========\nDONE");
}
