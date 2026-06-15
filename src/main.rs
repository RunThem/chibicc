// Commit: 84cfcaf98f3d19c8f0f316e22a61725ad201f0f6

#![allow(unused)]

mod ast;
mod parse;
mod tokenize;
mod utils;

fn main() {
  let tokenizer = tokenize::Tokenizer::new("hello.cxx");
  let mut program = parse::Program::new(tokenizer);

  program.parse();

  dbg!(&program.ast);
}
