// Commit: 3d8627719be00e39070eaca0ee5b599f2a877c5c

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
