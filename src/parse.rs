use crate::{
  ast::{
    Function, Node,
    NodeKind::{self, NdBlock},
    Obj, Program, TokenID,
  },
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
    self.consume("{");

    let body = self.compound_stmt();

    let mut func = Function {
      stack_size: 0,
      body: body,
      locals: self.locals.clone(),
    };

    self.asts = Mbox::new(func);
  }

  // stmp = expr-stmt
  //      | "return" expr ";"
  //      | "(" compound-stmt
  //      | "if" "(" expr ")" stmp ("else" stmp)?
  //      | "for" "(" expr-stmt expr? ";" expr? ")" stmp
  //      | "while" "(" expr ")" stmp
  fn stmt(&mut self) -> Mbox<Node> {
    if self.consume("return") {
      let node = Mbox::new(Node::from_unary(NodeKind::NdReturn, self.expr(), self.pos));

      self.expect(";");

      return node;
    }

    if self.consume("if") {
      self.expect("(");

      let cond = self.expr();

      self.expect(")");

      let then = self.stmt();

      let els = if self.consume("else") {
        self.stmt()
      } else {
        Mbox::nil()
      };

      let mut node = Node::from(NodeKind::NdIf, self.pos);

      node.cond = cond;
      node.then = then;
      node.els = els;

      return Mbox::new(node);
    }

    if self.consume("for") {
      self.expect("(");

      let init = self.expr_stmt();

      let cond = if !self.consume(";") {
        let node = self.expr();

        self.expect(";");

        node
      } else {
        Mbox::nil()
      };

      let inc = if !self.consume(")") {
        let node = self.expr();

        self.expect(";");

        node
      } else {
        Mbox::nil()
      };

      let body = self.stmt();

      let mut node = Node::from(NodeKind::NdFor, self.pos);

      node.init = init;
      node.cond = cond;
      node.inc = inc;
      node.then = body;

      return Mbox::new(node);
    }

    if self.consume("while") {
      self.expect("(");

      let cond = self.expr();

      self.expect(")");

      let body = self.stmt();

      let mut node = Node::from(NodeKind::NdFor, self.pos);

      node.cond = cond;
      node.then = body;

      return Mbox::new(node);
    }

    if self.consume("{") {
      return self.compound_stmt();
    }

    self.expr_stmt()
  }

  // compound-stmt = stmt* "}"
  fn compound_stmt(&mut self) -> Mbox<Node> {
    let mut node = Node::from(NdBlock, self.pos);

    while !self.consume("}") {
      node.body.push(self.stmt());
    }

    Mbox::new(node)
  }

  // expr-stmt = expr? ";"
  fn expr_stmt(&mut self) -> Mbox<Node> {
    if self.consume(";") {
      return Mbox::new(Node::from(NodeKind::NdBlock, self.pos));
    }

    let node = Mbox::new(Node::from_unary(
      NodeKind::NdExprStmp,
      self.expr(),
      self.pos,
    ));

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
      return Mbox::new(Node::from_binary(
        NodeKind::NdAssign,
        node,
        self.assign(),
        self.pos,
      ));
    }

    node
  }

  // equality = relational ("==" relational | "!=" relational)*
  fn equality(&mut self) -> Mbox<Node> {
    let mut node = self.relational();

    loop {
      if self.consume("==") {
        node = Mbox::new(Node::from_binary(
          NodeKind::NdEq,
          node,
          self.relational(),
          self.pos,
        ));
        continue;
      }

      if self.consume("!=") {
        node = Mbox::new(Node::from_binary(
          NodeKind::NdNe,
          node,
          self.relational(),
          self.pos,
        ));
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
        node = Mbox::new(Node::from_binary(
          NodeKind::NdLt,
          node,
          self.add(),
          self.pos,
        ));
        continue;
      }

      if self.consume("<=") {
        node = Mbox::new(Node::from_binary(
          NodeKind::NdLe,
          node,
          self.add(),
          self.pos,
        ));
        continue;
      }

      if self.consume(">") {
        node = Mbox::new(Node::from_binary(
          NodeKind::NdLt,
          self.add(),
          node,
          self.pos,
        ));
        continue;
      }

      if self.consume(">=") {
        node = Mbox::new(Node::from_binary(
          NodeKind::NdLe,
          self.add(),
          node,
          self.pos,
        ));
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
        node = Mbox::new(Node::from_binary(
          NodeKind::NdAdd,
          node,
          self.mul(),
          self.pos,
        ));
        continue;
      }

      if self.consume("-") {
        node = Mbox::new(Node::from_binary(
          NodeKind::NdSub,
          node,
          self.mul(),
          self.pos,
        ));
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
        node = Mbox::new(Node::from_binary(
          NodeKind::NdMul,
          node,
          self.unary(),
          self.pos,
        ));
        continue;
      }

      if self.consume("/") {
        node = Mbox::new(Node::from_binary(
          NodeKind::NdDiv,
          node,
          self.unary(),
          self.pos,
        ));
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
      return Mbox::new(Node::from_unary(NodeKind::NdNeg, self.unary(), self.pos));
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
      let mut node = Node::from(NodeKind::NdNum, self.pos);

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

      let node = Node::from_obj(obj_id, self.pos);

      self.pos += 1;

      return Mbox::new(node);
    }

    panic!("expected 按expression");
  }
}
