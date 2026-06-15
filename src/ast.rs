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

  pub token_id: TokenID,
}

impl Node {
  pub fn from(kind: NodeKind) -> Self {
    Self {
      kind,
      lhs: Mbox::nil(),
      rhs: Mbox::nil(),
      token_id: usize::MAX,
    }
  }

  pub fn from_binary(kind: NodeKind, lhs: Mbox<Node>, rhs: Mbox<Node>) -> Self {
    Self {
      kind,
      lhs,
      rhs,
      token_id: usize::MAX,
    }
  }

  pub fn from_unary(kind: NodeKind, expr: Mbox<Node>) -> Self {
    Self {
      kind,
      lhs: expr,
      rhs: Mbox::nil(),
      token_id: usize::MAX,
    }
  }

  pub fn from_token(kind: NodeKind, token_id: TokenID) -> Self {
    Self {
      kind,
      lhs: Mbox::nil(),
      rhs: Mbox::nil(),
      token_id,
    }
  }
}

pub struct Program {
  pub tokenizer: Tokenizer,
  pub pos: TokenID,
  pub asts: Vec<Mbox<Node>>,
}

impl Program {
  fn dump_ast(tokens: &Vec<Token>, ast: &Node, retract: usize) {
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
        Self::dump_ast(tokens, &ast.lhs, retract + 2);
        println!("{}rhs:", " ".repeat(retract));
        Self::dump_ast(tokens, &ast.rhs, retract + 2);
      }

      NodeKind::NdNeg => {
        println!("{}{}:", " ".repeat(retract), "-");
        Self::dump_ast(tokens, &ast.lhs, retract + 2);
      }

      NodeKind::NdEq | NodeKind::NdNe | NodeKind::NdLt | NodeKind::NdLe => {
        let op = ["==", "!=", "<", "<="][ast.kind as usize - NodeKind::NdEq as usize];

        println!("{}{}:", " ".repeat(retract), op);

        println!("{}lhs:", " ".repeat(retract));
        Self::dump_ast(tokens, &ast.lhs, retract + 2);
        println!("{}rhs:", " ".repeat(retract));
        Self::dump_ast(tokens, &ast.rhs, retract + 2);
      }

      NodeKind::NdAssign => {
        println!("{}=:", " ".repeat(retract));

        println!("{}lhs:", " ".repeat(retract));
        Self::dump_ast(tokens, &ast.lhs, retract + 2);
        println!("{}rhs:", " ".repeat(retract));
        Self::dump_ast(tokens, &ast.rhs, retract + 2);
      }

      NodeKind::NdExprStmp => {
        println!("{}ExprStmp:", " ".repeat(retract));
        Self::dump_ast(tokens, &ast.lhs, retract + 2);
      }

      NodeKind::NdVar => {
        let token = &tokens[ast.token_id];
        println!("{}NdVar: {}", " ".repeat(retract), token.tok);
      }

      _ => unreachable!(),
    }
  }

  pub fn dump(&self) {
    for (ast, idx) in self.asts.iter().zip(0..) {
      println!("AST #{}:", idx);
      Self::dump_ast(&self.tokenizer.tokens, ast, 2);
    }
  }
}
