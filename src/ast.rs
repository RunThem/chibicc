use crate::{
  tokenize::{Token, Tokenizer},
  utils::Mbox,
};

// 抽象语法树节点类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
  NdNum,      // Integer
  NdAdd,      // +
  NdSub,      // -
  NdMul,      // *
  NdDiv,      // /
  NdNeg,      // unary -
  NdEq,       // ==
  NdNe,       // !=
  NdLt,       // <
  NdLe,       // <=
  NdAssign,   // =
  NdReturn,   // return
  NdIf,       // if
  NdBlock,    // { ... }
  NdExprStmp, // Expression statement
  NdVar,      // Variable
}

pub type TokenID = usize;

// 抽象语法树节点
#[derive(Debug, Clone)]
pub struct Node {
  pub kind: NodeKind,

  pub lhs: Mbox<Node>,
  pub rhs: Mbox<Node>,

  // If 语句的子节点
  pub cond: Mbox<Node>,
  pub then: Mbox<Node>,
  pub els: Mbox<Node>,

  // Block 语句的子节点
  pub body: Vec<Mbox<Node>>,

  pub obj: ObjID,

  pub token_id: TokenID,
}

impl Node {
  pub fn from(kind: NodeKind) -> Self {
    Self {
      kind,
      lhs: Mbox::nil(),
      rhs: Mbox::nil(),
      cond: Mbox::nil(),
      then: Mbox::nil(),
      els: Mbox::nil(),

      body: Vec::new(),

      obj: usize::MAX,
      token_id: usize::MAX,
    }
  }

  pub fn from_binary(kind: NodeKind, lhs: Mbox<Node>, rhs: Mbox<Node>) -> Self {
    let mut this = Self::from(kind);
    this.lhs = lhs;
    this.rhs = rhs;
    this
  }

  pub fn from_unary(kind: NodeKind, expr: Mbox<Node>) -> Self {
    let mut this = Self::from(kind);
    this.lhs = expr;
    this
  }

  pub fn from_token(kind: NodeKind, token_id: TokenID) -> Self {
    let mut this = Self::from(kind);
    this.token_id = token_id;
    this
  }

  pub fn from_obj(obj: ObjID) -> Self {
    let mut this = Self::from(NodeKind::NdVar);
    this.obj = obj;
    this
  }
}

pub type ObjID = usize;

#[derive(Debug, Clone, Copy)]
pub struct Obj {
  pub(crate) token_id: TokenID,
  pub(crate) offset: i32,
}

#[derive(Debug)]
pub struct Function {
  pub(crate) stack_size: i32,
  pub(crate) locals: Vec<Obj>,
  pub(crate) body: Mbox<Node>,
}

pub struct Program {
  pub tokenizer: Tokenizer,
  pub pos: TokenID,
  pub locals: Vec<Obj>,
  pub asts: Mbox<Function>,
}

impl Program {
  fn dump_ast(tokens: &Vec<Token>, objs: &Vec<Obj>, ast: &Node, retract: usize) {
    match ast.kind {
      NodeKind::NdNum => {
        let token = &tokens[ast.token_id];
        println!("{}NdNum: {}", " ".repeat(retract), token.tok);
      }

      NodeKind::NdAdd | NodeKind::NdSub | NodeKind::NdMul | NodeKind::NdDiv => {
        let op = match ast.kind {
          NodeKind::NdAdd => "+",
          NodeKind::NdSub => "-",
          NodeKind::NdMul => "*",
          NodeKind::NdDiv => "/",
          _ => unreachable!(),
        };

        println!("{}{}:", " ".repeat(retract), op);

        println!("{}lhs:", " ".repeat(retract));
        Self::dump_ast(tokens, objs, &ast.lhs, retract + 2);
        println!("{}rhs:", " ".repeat(retract));
        Self::dump_ast(tokens, objs, &ast.rhs, retract + 2);
      }

      NodeKind::NdNeg => {
        println!("{}{}:", " ".repeat(retract), "-");
        Self::dump_ast(tokens, objs, &ast.lhs, retract + 2);
      }

      NodeKind::NdEq | NodeKind::NdNe | NodeKind::NdLt | NodeKind::NdLe => {
        let op = ["==", "!=", "<", "<="][ast.kind as usize - NodeKind::NdEq as usize];

        println!("{}{}:", " ".repeat(retract), op);

        println!("{}lhs:", " ".repeat(retract));
        Self::dump_ast(tokens, objs, &ast.lhs, retract + 2);
        println!("{}rhs:", " ".repeat(retract));
        Self::dump_ast(tokens, objs, &ast.rhs, retract + 2);
      }

      NodeKind::NdAssign => {
        println!("{}=:", " ".repeat(retract));

        println!("{}lhs:", " ".repeat(retract));
        Self::dump_ast(tokens, objs, &ast.lhs, retract + 2);
        println!("{}rhs:", " ".repeat(retract));
        Self::dump_ast(tokens, objs, &ast.rhs, retract + 2);
      }

      NodeKind::NdExprStmp => {
        println!("{}ExprStmp:", " ".repeat(retract));
        Self::dump_ast(tokens, objs, &ast.lhs, retract + 2);
      }

      NodeKind::NdVar => {
        let token = &tokens[objs[ast.obj].token_id];
        println!("{}NdVar: {}", " ".repeat(retract), token.tok);
      }

      NodeKind::NdReturn => {
        println!("{}return:", " ".repeat(retract));

        Self::dump_ast(tokens, objs, &ast.lhs, retract + 2);
      }

      NodeKind::NdBlock => {
        println!("{}Block:", " ".repeat(retract));

        for stmt in &ast.body {
          Self::dump_ast(tokens, objs, stmt, retract + 2);
        }
      }

      NodeKind::NdIf => {
        println!("{}if:", " ".repeat(retract));

        println!("{}cond:", " ".repeat(retract));
        Self::dump_ast(tokens, objs, &ast.cond, retract + 2);

        println!("{}then:", " ".repeat(retract));
        Self::dump_ast(tokens, objs, &ast.then, retract + 2);

        if !ast.els.is_nil() {
          println!("{}else:", " ".repeat(retract));
          Self::dump_ast(tokens, objs, &ast.els, retract + 2);
        }
      }

      _ => unreachable!(),
    }
  }

  pub fn dump(&self) {
    println!("stack-size: {}", self.asts.stack_size);

    for local in &self.asts.locals {
      let token = &self.tokenizer.tokens[local.token_id];
      println!("local: {} (offset {})", token.tok, local.offset);
    }

    Self::dump_ast(
      &self.tokenizer.tokens,
      &self.asts.locals,
      &self.asts.body,
      0,
    );
  }
}
