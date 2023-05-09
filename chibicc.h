#include <assert.h>
#include <ctype.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct Node Node;

//
// tokenizer.c
//

typedef enum {
  TK_IDENT,    // Identifiers, 标识符
  TK_PUNCT,    // Keywords or punctuators, 关键字或标点符号
  TK_KEYWORD,  // keywords
  TK_NUM,      // Numeric literals, 数字字符
  TK_EOF,      // End-of-file markers, 文件结束标记
} TokenKind;

// Token type
typedef struct Token Token;
struct Token {
  TokenKind kind;  // Token kind, Token 种类
  Token* next;     // Next token, 下一个 Token
  int val;         // If kind is TK_NUM, 如果类型为 TK_NUM, 其值
  char* loc;       // Token, location, Token 位置
  int len;         // Token length, Token 长度
};

void error(char* fmt, ...);
void verror_at(char* loc, char* fmt, va_list ap);
void error_at(char* loc, char* fmt, ...);
void error_tok(Token* tok, char* fmt, ...);
bool equal(Token* tok, char* op);
Token* skip(Token* tok, char* s);
Token* tokenize(char* input);
void show_tokens(Token* tok);

//
// parser.c
//

// Local variable
typedef struct Obj Obj;
struct Obj {
  Obj* next;
  char* name;  // variable name, 变量名
  int offset;  // Offset from RBP
};

// Function
typedef struct Function Function;
struct Function {
  Node* body;
  Obj* locals;
  int stack_size;
};

// AST node
typedef enum {
  ND_ADD,        // +
  ND_SUB,        // -
  ND_MUL,        // *
  ND_DIV,        // /
  ND_NEG,        // unary -
  ND_EQ,         // ==
  ND_NE,         // !=
  ND_LT,         // <
  ND_LE,         // <=
  ND_ASSIGN,     // =
  ND_ADDR,       // unary &
  ND_DEREF,      // unary *
  ND_RETURN,     // "return"
  ND_IF,         // "if"
  ND_FOR,        // "for" or "while"
  ND_BLOCK,      // { ... }
  ND_EXPR_STMT,  // Expression statement, 表达式语句
  ND_VAR,        // Variable
  ND_NUM,        // Integer
} NodeKind;

// AST node type
struct Node {
  NodeKind kind;  // Node kind
  Node* next;     // Next node
  Token* tok;     // Representative token, 代表当前节点的 Token

  Node* lhs;  // Left-hand side, 左侧
  Node* rhs;  // Right-hand side, 右侧

  // "if" or "for" statement
  Node* cond;
  Node* then;
  Node* els;
  Node* init;
  Node* inc;

  // Block
  Node* body;

  Obj* var;  // Used if kind == ND_VAR
  int val;   // Used if kind == ND_NUM
};

typedef struct Trunk {
  struct Trunk* prev;
  char* str;
} Trunk;

Function* parse(Token* tok);

void show_trees(Node* root, Trunk* prev, bool is_left);

//
// codegen.c
//

void codegen(Function* prog);
