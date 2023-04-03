#include <ctype.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

//
// Tokenizer
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

// input string
// 输入字符串
static char* current_input;

// Reports an error and exit
// 报告错误并退出
static void error(char* fmt, ...) {
  va_list ap;
  va_start(ap, fmt);
  vfprintf(stderr, fmt, ap);
  fprintf(stderr, "\n");
  va_end(ap);
  exit(1);
}

// Reports an error location and exit
// 报告错误位置并退出
static void verror_at(char* loc, char* fmt, va_list ap) {
  int pos = loc - current_input;
  fprintf(stderr, "%s\n", current_input);
  fprintf(stderr, "%*s", pos, "");  // print pos spaces, 输出一定空格
  fprintf(stderr, "^ ");
  vfprintf(stderr, fmt, ap);
  fprintf(stderr, "\n");

  exit(1);
}

static void error_at(char* loc, char* fmt, ...) {
  va_list ap;
  va_start(ap, fmt);
  verror_at(loc, fmt, ap);
}

static void error_tok(Token* tok, char* fmt, ...) {
  va_list ap;
  va_start(ap, fmt);
  verror_at(tok->loc, fmt, ap);
}

// Consumes the current token if it matches `s`
// 如果当前 Token 与 `s` 匹配, 则使用当前 Token
static bool equal(Token* tok, char* op) {
  return memcmp(tok->loc, op, tok->len) == 0 && op[tok->len] == '\0';
}

// Ensure that the current token is `s`
// 确保当前 Token 与 `s` 匹配
static Token* skip(Token* tok, char* s) {
  if (!equal(tok, s)) {
    error_tok(tok, "expected '%s'", s);
  }

  return tok->next;
}

// Ensure that the current token is TK_NUM
// 确保当前 Token 类型为 TK_NUM
static int get_number(Token* tok) {
  if (tok->kind != TK_NUM) {
    error_tok(tok, "expected a number");
  }

  return tok->val;
}

// Create a new token
// 创建一个 Token
static Token* new_token(TokenKind kind, char* start, char* end) {
  Token* tok = calloc(1, sizeof(Token));
  tok->kind  = kind;
  tok->len   = end - start;
  tok->loc   = strndup(start, tok->len);
  return tok;
}

// Tokenize `current_input` and returns new tokens
// 标记化 `current_input` 并返回新的 Token
static Token* tokenize(void) {
  char* p    = current_input;
  Token head = {};
  Token* cur = &head;

  while (*p) {
    // Skip whitespace characters
    // 跳过空白字符
    if (isspace(*p)) {
      p++;
      continue;
    }

    // Numeric literal
    // 数字字符
    if (isdigit(*p)) {
      cur = cur->next = new_token(TK_NUM, p, p);
      char* q         = p;
      cur->val        = strtoul(p, &p, 10);
      cur->len        = p - q;
      continue;
    }

    // Punctuator
    // 标点符号
    if (ispunct(*p)) {
      cur = cur->next = new_token(TK_PUNCT, p, p + 1);
      p++;
      continue;
    }

    error_at(p, "invalid token");
  }

  cur = cur->next = new_token(TK_EOF, p, p);
  return head.next;
}

static void token_dump(Token* tok) {
  while (tok->kind != TK_EOF) {
    switch (tok->kind) {
      case TK_PUNCT:
        printf("punct '%s'\n", tok->loc);
        break;
      case TK_NUM:
        printf("num %d\n", tok->val);
        break;
      default:
        return;
    }

    tok = tok->next;
  }

  printf("\n");
}

//
// Parser
//

typedef enum {
  ND_ADD,  // +
  ND_SUB,  // -
  ND_MUL,  // *
  ND_DIV,  // /
  ND_NUM,  // Integer
} NodeKind;

// AST node type
typedef struct Node Node;
struct Node {
  NodeKind kind;  // Node kind
  Node* lhs;      // Left-hand side, 左侧
  Node* rhs;      // Right-hand side, 右侧
  int val;        // Used if kind == ND_NUM
};

static Node* new_node(NodeKind kind) {
  Node* node = calloc(1, sizeof(Node));
  node->kind = kind;
  return node;
}

static Node* new_binary(NodeKind kind, Node* lhs, Node* rhs) {
  Node* node = new_node(kind);
  node->lhs  = lhs;
  node->rhs  = rhs;
  return node;
}

static Node* new_num(int val) {
  Node* node = new_node(ND_NUM);
  node->val  = val;
  return node;
}

static Node* expr(Token** rest, Token* tok);
static Node* mul(Token** rest, Token* tok);
static Node* primary(Token** rest, Token* tok);

// expr = mul ("+" mul | "-" mul)*
static Node* expr(Token** rest, Token* tok) {
  Node* node = mul(&tok, tok);

  for (;;) {
    if (equal(tok, "+")) {
      node = new_binary(ND_ADD, node, mul(&tok, tok->next));
      continue;
    }

    if (equal(tok, "-")) {
      node = new_binary(ND_SUB, node, mul(&tok, tok->next));
      continue;
    }

    *rest = tok;
    return node;
  }
}

// mul = primary ("*" primary | "/" primary)*
static Node* mul(Token** rest, Token* tok) {
  Node* node = primary(&tok, tok);

  for (;;) {
    if (equal(tok, "*")) {
      node = new_binary(ND_MUL, node, primary(&tok, tok->next));
      continue;
    }

    if (equal(tok, "/")) {
      node = new_binary(ND_DIV, node, primary(&tok, tok->next));
      continue;
    }

    *rest = tok;
    return node;
  }
}

// primary = "(" expr ")" | num
static Node* primary(Token** rest, Token* tok) {
  if (equal(tok, "(")) {
    Node* node = expr(&tok, tok->next);
    *rest      = skip(tok, ")");
    return node;
  }

  if (tok->kind == TK_NUM) {
    Node* node = new_num(tok->val);
    *rest      = tok->next;
    return node;
  }

  error_tok(tok, "expected an expression");
}

//
// Code generator
//

static int depth;

static void push(void) {
  printf("  push %%rax\n");
  depth++;
}

static void pop(char* arg) {
  printf("  pop %s\n", arg);
  depth--;
}

static void gen_expr(Node* node) {
  if (node->kind == ND_NUM) {
    printf("  mov $%d, %%rax\n", node->val);
    return;
  }

  gen_expr(node->rhs);
  push();
  gen_expr(node->lhs);
  pop("%rdi");

  switch (node->kind) {
    case ND_ADD:
      printf("  add %%rdi, %%rax\n");
      return;
    case ND_SUB:
      printf("  sub %%rdi, %%rax\n");
      return;
    case ND_MUL:
      printf("  imul %%rdi, %%rax\n");
      return;
    case ND_DIV:
      printf("  cqo\n");
      printf("  idiv %%rdi\n");
      return;
    default:
      return;
  }

  error("invalid expression");
}

int main(int argc, char** argv) {
  if (argc != 2) {
    error("%s: invalid number of arguments\n", argv[0]);
  }

  current_input = argv[1];
  Token* tok    = tokenize();
  // token_dump(tok);

  Node* node = expr(&tok, tok);

  if (tok->kind != TK_EOF) {
    error_tok(tok, "extra token");
  }

  printf("  .globl main\n");
  printf("main:\n");

  // Traverse the AST to emit assembly
  // 遍历 AST 输出汇编
  gen_expr(node);

  printf("  ret\n");

  return 0;
}
