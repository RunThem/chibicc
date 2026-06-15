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
      asts: Vec::new(),
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
    loop {
      if self.peek().kind == TokenKind::TkEof {
        break;
      }

      let ast = self.stmt();

      self.asts.push(ast);
    }
  }

  // stmp = expr-stmt
  fn stmt(&mut self) -> Mbox<Node> {
    self.expr_stmt()
  }

  // expr-stmp = expr ";"
  fn expr_stmt(&mut self) -> Mbox<Node> {
    let node = self.expr();

    self.expect(";");

    node
  }

  // expr = equality
  fn expr(&mut self) -> Mbox<Node> {
    self.equality()
  }

  // equality = relational ("==" relational | "!=" relational)*
  fn equality(&mut self) -> Mbox<Node> {
    let mut node = self.relational();

    loop {
      if self.consume("==") {
        node = Mbox::new(Node::from_binary(NodeKind::NdEq, node, self.relational()));
        continue;
      }

      if self.consume("!=") {
        node = Mbox::new(Node::from_binary(NodeKind::NdNe, node, self.relational()));
        continue;
      }

      break;
    }

    node
  }

  // relational = add ("<" add | "<=" add | ">" add | ">=" add)*
  fn relational(&mut self) -> Mbox<Node> {
    let mut node = self.add();

    loop {
      if self.consume("<") {
        node = Mbox::new(Node::from_binary(NodeKind::NdLt, node, self.add()));
        continue;
      }

      if self.consume("<=") {
        node = Mbox::new(Node::from_binary(NodeKind::NdLe, node, self.add()));
        continue;
      }

      if self.consume(">") {
        node = Mbox::new(Node::from_binary(NodeKind::NdLt, self.add(), node));
        continue;
      }

      if self.consume(">=") {
        node = Mbox::new(Node::from_binary(NodeKind::NdLe, self.add(), node));
        continue;
      }

      break;
    }

    node
  }

  // add = mul ("+" mul | "-" mul)*
  fn add(&mut self) -> Mbox<Node> {
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
      let node = self.add();

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
