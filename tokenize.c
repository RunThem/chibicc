#include "chibicc.h"

// input string
// 输入字符串
static char* current_input;

// Reports an error and exit
// 报告错误并退出
void error(char* fmt, ...) {
  va_list ap;
  va_start(ap, fmt);
  vfprintf(stderr, fmt, ap);
  fprintf(stderr, "\n");
  va_end(ap);
  exit(1);
}

// Reports an error location and exit
// 报告错误位置并退出
void verror_at(char* loc, char* fmt, va_list ap) {
  int pos = loc - current_input;
  fprintf(stderr, "%s\n", current_input);
  fprintf(stderr, "%*s", pos, "");  // print pos spaces, 输出一定空格
  fprintf(stderr, "^ ");
  vfprintf(stderr, fmt, ap);
  fprintf(stderr, "\n");

  exit(1);
}

void error_at(char* loc, char* fmt, ...) {
  va_list ap;
  va_start(ap, fmt);
  verror_at(loc, fmt, ap);
}

void error_tok(Token* tok, char* fmt, ...) {
  va_list ap;
  va_start(ap, fmt);
  verror_at(tok->loc, fmt, ap);
}

// Consumes the current token if it matches `s`
// 如果当前 Token 与 `s` 匹配, 则使用当前 Token
bool equal(Token* tok, char* op) {
  return memcmp(tok->loc, op, tok->len) == 0 && op[tok->len] == '\0';
}

// Ensure that the current token is `s`
// 确保当前 Token 与 `s` 匹配
Token* skip(Token* tok, char* s) {
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

static bool startswitch(char* p, char* q) {
  return strncmp(p, q, strlen(q)) == 0;
}

// Read a punctuator token from p and returns its length
// 从 `p` 中读取一个标点符号 Token, 并返回其长度
static int read_punct(char* p) {
  if (startswitch(p, "==") || startswitch(p, "!=") || startswitch(p, "<=") ||
      startswitch(p, ">=")) {
    return 2;
  }

  return ispunct(*p) ? 1 : 0;
}

// Tokenize `current_input` and returns new tokens
// 标记化 `current_input` 并返回新的 Token
Token* tokenize(char* p) {
  current_input = p;
  Token head    = {};
  Token* cur    = &head;

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

    // Identifier
    if ('a' <= *p && *p <= 'z') {
      cur = cur->next = new_token(TK_IDENT, p, p + 1);
      p++;
      continue;
    }

    // Punctuator
    // 标点符号
    int punct_len = read_punct(p);
    if (punct_len) {
      cur = cur->next = new_token(TK_PUNCT, p, p + punct_len);
      p += cur->len;
      continue;
    }

    error_at(p, "invalid token");
  }

  cur = cur->next = new_token(TK_EOF, p, p);
  return head.next;
}

void token_dump(Token* tok) {
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
