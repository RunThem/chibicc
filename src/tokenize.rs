use core::cmp::Eq;
use std::{ops::Index, ops::Range};

#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
  TkPunct,
  TkNum,
  TkEof,
}

#[derive(Debug, Eq)]
pub struct Token {
  pub kind: TokenKind,
  pub tok: String,
  pub span: Range<usize>,
}

impl PartialEq for Token {
  fn eq(&self, other: &Self) -> bool {
    self.kind == other.kind && self.tok == other.tok
  }
}

impl PartialEq<&str> for Token {
  fn eq(&self, other: &&str) -> bool {
    &self.tok == other
  }
}

impl PartialEq<&str> for &Token {
  fn eq(&self, other: &&str) -> bool {
    &self.tok == other
  }
}

#[derive(Debug)]
pub struct Tokenizer {
  pub path: String,
  pub context: String,
  pub tokens: Vec<Token>,
}

impl Index<usize> for Tokenizer {
  type Output = Token;

  fn index(&self, index: usize) -> &Self::Output {
    &self.tokens[index]
  }
}

impl Tokenizer {
  // 将 clex::Lexeme 转换为 Token
  fn transform(lexeme: clex::Lexeme) -> Token {
    let kind = match lexeme.token {
      clex::Token::Int => TokenKind::TkNum,
      clex::Token::Symbol => TokenKind::TkPunct,
      _ => TokenKind::TkEof,
    };

    let token = Token {
      kind: kind,
      tok: lexeme.slice.to_string(),
      span: lexeme.span.into(),
    };

    token
  }

  pub fn new(path: &str) -> Self {
    let context = std::fs::read_to_string(path).expect("failed to read file");

    let mut tokens = clex::Lexer::from(&context[..])
      .map(|lexeme| Self::transform(lexeme))
      .collect::<Vec<Token>>();

    tokens.push(Token {
      kind: TokenKind::TkEof,
      tok: String::default(),
      span: 0..0,
    });

    Self {
      path: path.into(),
      context,
      tokens,
    }
  }
}
