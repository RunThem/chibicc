#include <ctype.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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
  tok->loc   = start;
  tok->len   = end - start;
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
    if (*p == '+' || *p == '-') {
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
        printf("punct '%s'\n", strndup(tok->loc, tok->len));
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

int main(int argc, char** argv) {
  if (argc != 2) {
    error("%s: invalid number of arguments\n", argv[0]);
  }

  current_input = argv[1];
  Token* tok    = tokenize();
  // token_dump(tok);

  printf("  .globl main\n");
  printf("main:\n");

  // The first token must be a number
  // 第一个 Token 必须是数字
  printf("  mov $%d, %%rax\n", get_number(tok));
  tok = tok->next;

  // ... followed by either `+ <number>` or `- <number>`
  // 后面跟着 `+ <number>` 或 `- <number>`
  while (tok->kind != TK_EOF) {
    if (equal(tok, "+")) {
      printf("  add $%d, %%rax\n", get_number(tok->next));
      tok = tok->next->next;
      continue;
    }

    tok = skip(tok, "-");
    printf("  sub $%d, %%rax\n", get_number(tok));
    tok = tok->next;
  }

  printf("  ret\n");

  return 0;
}
