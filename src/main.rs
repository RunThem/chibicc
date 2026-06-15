// Commit: 725badfb494544b7c7f1d4c4690b9bc033c6d051

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
