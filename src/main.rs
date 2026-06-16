// Commit: 1f3eb34f637520b01e6b8cd10a9026d05036db6d

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
