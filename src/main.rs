// Commit: 1f9f3adf324af1432a380b41c7690834e649e346

#![allow(unused)]

mod ast;
mod parse;
mod tokenize;
mod utils;

fn main() {
  let tokenizer = tokenize::Tokenizer::new("hello.cxx");
  let mut program = ast::Program::new(tokenizer);

  program.parse();

  program.dump();
}
