use crate::{tokenize::Token, utils::Mbox};

// 抽象语法树节点类型
#[derive(Debug, PartialEq, Eq)]
pub enum NodeKind {
  NdNum, // Integer
  NdAdd, // +
  NdSub, // -
  NdMul, // *
  NdDiv, // /
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

  pub fn from_token(kind: NodeKind, token_id: TokenID) -> Self {
    Self {
      kind,
      lhs: Mbox::nil(),
      rhs: Mbox::nil(),
      token_id,
    }
  }
}
