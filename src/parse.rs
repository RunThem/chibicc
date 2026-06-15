use crate::{
  ast::{Node, NodeKind, Program, TokenID},
  tokenize::{
    Token,
    TokenKind::{self, TkNum},
    Tokenizer,
  },
  utils::Mbox,
};

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

  // mul = unary ("*" unary | "/" unary)*
  fn mul(&mut self) -> Mbox<Node> {
    let mut node = self.unary();

    loop {
      if self.consume("*") {
        node = Mbox::new(Node::from_binary(NodeKind::NdMul, node, self.unary()));
        continue;
      }

      if self.consume("/") {
        node = Mbox::new(Node::from_binary(NodeKind::NdDiv, node, self.unary()));
        continue;
      }

      break;
    }

    node
  }

  // unary = ("+" | "-") unary
  //       | primary
  fn unary(&mut self) -> Mbox<Node> {
    if self.consume("+") {
      return self.unary();
    }

    if self.consume("-") {
      return Mbox::new(Node::from_unary(NodeKind::NdNeg, self.unary()));
    }

    self.primary()
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
