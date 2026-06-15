use crate::{
  ast::{Node, NodeKind, TokenID},
  tokenize::{
    Token,
    TokenKind::{self, TkNum},
    Tokenizer,
  },
  utils::Mbox,
};

pub struct Program {
  pub tokenizer: Tokenizer,
  pub pos: TokenID,
  pub ast: Mbox<Node>,
}

impl Program {
  pub fn new(tokenizer: Tokenizer) -> Self {
    Self {
      tokenizer,
      pos: 0,
      ast: Mbox::nil(),
    }
  }

  fn peek(&self) -> &Token {
    &self.tokenizer[self.pos]
  }

  fn consume(&mut self, expected: &str) -> bool {
    if self.peek() == expected {
      self.pos += 1;
      true
    } else {
      false
    }
  }

  fn expect(&mut self, expected: &str) {
    if !self.consume(expected) {
      panic!(
        "Expected token '{}', but found '{}'",
        expected,
        self.peek().tok
      );
    }
  }

  pub fn parse(&mut self) {
    self.ast = self.expr();
  }

  // expr = mul ("+" mul | "-" mul)*
  fn expr(&mut self) -> Mbox<Node> {
    let mut node = self.mul();

    loop {
      if self.consume("+") {
        node = Mbox::new(Node::from_binary(NodeKind::NdAdd, node, self.mul()));
        continue;
      }

      if self.consume("-") {
        node = Mbox::new(Node::from_binary(NodeKind::NdSub, node, self.mul()));
        continue;
      }

      break;
    }

    node
  }

  // mul = primary ("*" primary | "/" primary)*
  fn mul(&mut self) -> Mbox<Node> {
    let mut node = self.primary();

    loop {
      if self.consume("*") {
        node = Mbox::new(Node::from_binary(NodeKind::NdMul, node, self.primary()));
        continue;
      }

      if self.consume("/") {
        node = Mbox::new(Node::from_binary(NodeKind::NdDiv, node, self.primary()));
        continue;
      }

      break;
    }

    node
  }

  // primary = "(" expr ")" | num
  fn primary(&mut self) -> Mbox<Node> {
    if self.consume("(") {
      let node = self.expr();

      self.expect(")");

      return node;
    }

    if let token = self.peek()
      && token.kind == TkNum
    {
      let mut node = Node::from_token(NodeKind::NdNum, self.pos);

      self.pos += 1;

      return Mbox::new(node);
    }

    panic!("expected 按expression");
  }
}
