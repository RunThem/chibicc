#include <assert.h>
#include <ctype.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

//
// tokenizer.c
//

typedef enum {
  TK_PUNCT,  // Punctuators, 标点符号
  TK_NUM,    // Numeric literals, 数字字符
  TK_EOF,    // End-of-file markers, 文件结束标记
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
void token_dump(Token* tok);

//
// parser.c
//

typedef enum {
  ND_ADD,         // +
  ND_SUB,         // -
  ND_MUL,         // *
  ND_DIV,         // /
  ND_NEG,         // unary -
  ND_EQ,          // ==
  ND_NE,          // !=
  ND_LT,          // <
  ND_LE,          // <=
  ND_LEXPR_STMT,  // Expression statement, 表达式语句
  ND_NUM,         // Integer
} NodeKind;

// AST node type
typedef struct Node Node;
struct Node {
  NodeKind kind;  // Node kind
  Node* next;     // Next node
  Node* lhs;      // Left-hand side, 左侧
  Node* rhs;      // Right-hand side, 右侧
  int val;        // Used if kind == ND_NUM
};

Node* parse(Token* tok);

//
// codegen.c
//

void codegen(Node* node);
