use crate::{
  ast::{Function, Node, NodeKind, Obj, Program, TokenID},
  tokenize::{
    Token,
    TokenKind::{self, TkIdent, TkNum},
    Tokenizer,
  },
  utils::Mbox,
};

impl Program {
  pub fn new(tokenizer: Tokenizer) -> Self {
    Self {
      tokenizer,
      pos: 0,
      asts: Mbox::nil(),
      locals: Vec::new(),
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
    let mut func = Function {
      stack_size: 0,
      locals: Vec::new(),
      body: Vec::new(),
    };

    loop {
      if self.peek().kind == TokenKind::TkEof {
        break;
      }

      let ast = self.stmt();

      func.body.push(ast);
    }

    func.locals = self.locals.clone();

    self.asts = Mbox::new(func);
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

  // expr = assign
  fn expr(&mut self) -> Mbox<Node> {
    self.assign()
  }

  // assign = equality ("=" assign)?
  fn assign(&mut self) -> Mbox<Node> {
    let node = self.equality();

    if self.consume("=") {
      return Mbox::new(Node::from_binary(NodeKind::NdAssign, node, self.assign()));
    }

    node
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

  // primary = "(" expr ")" | num | ident
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

    if let token = self.peek()
      && token.kind == TkIdent
    {
      let var = self
        .locals
        .iter()
        .position(|o| self.tokenizer[o.token_id].tok == token.tok);

      let obj_id = match var {
        Some(idx) => idx,
        None => {
          self.locals.push(Obj {
            token_id: self.pos,
            offset: 0,
          });

          self.locals.len() - 1
        }
      };

      let node = Node::from_obj(obj_id);

      self.pos += 1;

      return Mbox::new(node);
    }

    panic!("expected 按expression");
  }
}
