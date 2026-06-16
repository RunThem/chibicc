// Commit: 18ac283a5d19c19f1e1a7020a50fe34c2160a0f8

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
