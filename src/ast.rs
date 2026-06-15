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
  NdExprStmp, // Expression statement
  NdVar,      // Variable
}

pub type TokenID = usize;

// 抽象语法树节点
#[derive(Debug)]
pub struct Node {
  pub kind: NodeKind,

  pub lhs: Mbox<Node>,
  pub rhs: Mbox<Node>,

  pub obj: ObjID,

  pub token_id: TokenID,
}

impl Node {
  pub fn from(kind: NodeKind) -> Self {
    Self {
      kind,
      lhs: Mbox::nil(),
      rhs: Mbox::nil(),
      obj: usize::MAX,
      token_id: usize::MAX,
    }
  }

  pub fn from_binary(kind: NodeKind, lhs: Mbox<Node>, rhs: Mbox<Node>) -> Self {
    Self {
      kind,
      lhs,
      rhs,
      obj: usize::MAX,
      token_id: usize::MAX,
    }
  }

  pub fn from_unary(kind: NodeKind, expr: Mbox<Node>) -> Self {
    Self {
      kind,
      lhs: expr,
      rhs: Mbox::nil(),
      obj: usize::MAX,
      token_id: usize::MAX,
    }
  }

  pub fn from_token(kind: NodeKind, token_id: TokenID) -> Self {
    Self {
      kind,
      lhs: Mbox::nil(),
      rhs: Mbox::nil(),
      obj: usize::MAX,
      token_id,
    }
  }

  pub fn from_obj(obj: ObjID) -> Self {
    Self {
      kind: NodeKind::NdVar,
      lhs: Mbox::nil(),
      rhs: Mbox::nil(),
      obj,
      token_id: usize::MAX,
    }
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
  pub(crate) body: Vec<Mbox<Node>>,
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

      _ => unreachable!(),
    }
  }

  pub fn dump(&self) {
    println!("stack-size: {}", self.asts.stack_size);

    for local in &self.asts.locals {
      let token = &self.tokenizer.tokens[local.token_id];
      println!("local: {} (offset {})", token.tok, local.offset);
    }

    for (i, ast) in self.asts.body.iter().enumerate() {
      println!("# {}:", i);
      Self::dump_ast(&self.tokenizer.tokens, &self.asts.locals, ast, 0);
    }
  }
}
