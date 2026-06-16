# Pratt Parsing 改写方案

## 1. 概述

将 `parse.c` 中 12 个几乎相同的二元运算符解析函数(`assign` 到 `mul`,共 ~320 行)替换为一个 Pratt 循环 + 优先级表(~100 行).

**不改动的部分**:`cast()`,`unary()`,`postfix()`,`primary()`,`eval`/`eval2`/`eval_double`/`is_const_expr`/`const_expr`(编译期求值器), 所有声明解析, initializer 解析.

## 2. 需要删除的函数

以下 12 个函数全部删除:

```
assign()        (第 2145-2183 行)
conditional()   (第 2186-2212 行)
logor()         (第 2215-2223 行)
logand()        (第 2226-2234 行)
bitor()         (第 2237-2245 行)
bitxor()        (第 2248-2256 行)
bitand()        (第 2259-2267 行)
equality()      (第 2270-2289 行)
relational()    (第 2292-2321 行)
shift()         (第 2324-2343 行)
add()           (第 2417-2436 行)
mul()           (第 2439-2463 行)
```

共删除约 320 行.

## 3. 需要新增的代码

### 3.1 优先级定义

```c
// Pratt 解析优先级级别.
// 数字越大, 绑定力越强(优先级越高).
// 左结合运算符: 右绑定力 = 左绑定力
// 右结合运算符: 右绑定力 = 左绑定力 - 1
//
// 这些数字替代了原来通过函数调用链编码的优先级关系.
enum {
  PREC_ASSIGN  = 2,   // = += -= *= /= %= &= |= ^= <<= >>=
  PREC_TERNARY = 3,   // ?:
  PREC_COMMA   = 4,   // ,
  PREC_LOGOR   = 5,   // ||
  PREC_LOGAND  = 6,   // &&
  PREC_BITOR   = 7,   // |
  PREC_BITXOR  = 8,   // ^
  PREC_BITAND  = 9,   // &
  PREC_EQ      = 10,  // == !=
  PREC_REL     = 11,  // < <= > >=
  PREC_SHIFT   = 12,  // << >>
  PREC_ADD     = 13,  // + -
  PREC_MUL     = 14,  // * / %
};
```

### 3.2 查表函数

```c
// 将二元运算符 token 映射到其左绑定力(left binding power).
// 如果 token 不是 Pratt 处理的二元运算符, 返回 0.
static int get_lbp(Token *tok) {
  if (equal(tok, "*"))  return PREC_MUL;
  if (equal(tok, "/"))  return PREC_MUL;
  if (equal(tok, "%"))  return PREC_MUL;
  if (equal(tok, "+"))  return PREC_ADD;
  if (equal(tok, "-"))  return PREC_ADD;
  if (equal(tok, "<<")) return PREC_SHIFT;
  if (equal(tok, ">>")) return PREC_SHIFT;
  if (equal(tok, "<"))  return PREC_REL;
  if (equal(tok, "<=")) return PREC_REL;
  if (equal(tok, ">"))  return PREC_REL;
  if (equal(tok, ">=")) return PREC_REL;
  if (equal(tok, "==")) return PREC_EQ;
  if (equal(tok, "!=")) return PREC_EQ;
  if (equal(tok, "&"))  return PREC_BITAND;
  if (equal(tok, "^"))  return PREC_BITXOR;
  if (equal(tok, "|"))  return PREC_BITOR;
  if (equal(tok, "&&")) return PREC_LOGAND;
  if (equal(tok, "||")) return PREC_LOGOR;
  if (equal(tok, ","))  return PREC_COMMA;
  return 0;
}

// 将赋值运算符 token 映射到对应的二元 AST 节点类型.
// 对于普通的 "=" 返回 0(特殊处理为 ND_ASSIGN).
static NodeKind get_assign_op(Token *tok) {
  if (equal(tok, "="))    return 0;
  if (equal(tok, "+="))   return ND_ADD;
  if (equal(tok, "-="))   return ND_SUB;
  if (equal(tok, "*="))   return ND_MUL;
  if (equal(tok, "/="))   return ND_DIV;
  if (equal(tok, "%="))   return ND_MOD;
  if (equal(tok, "&="))   return ND_BITAND;
  if (equal(tok, "|="))   return ND_BITOR;
  if (equal(tok, "^="))   return ND_BITXOR;
  if (equal(tok, "<<="))  return ND_SHL;
  if (equal(tok, ">>="))  return ND_SHR;
  error_tok(tok, "未知的赋值运算符");
}

// 检查当前 token 是否是赋值运算符.
static bool is_assign_op(Token *tok) {
  return equal(tok, "=") ||
         equal(tok, "+=")  || equal(tok, "-=")  ||
         equal(tok, "*=")  || equal(tok, "/=")  ||
         equal(tok, "%=")  || equal(tok, "&=")  ||
         equal(tok, "|=")  || equal(tok, "^=")  ||
         equal(tok, "<<=") || equal(tok, ">>=");
}

// 将二元运算符 token 映射到其 AST 节点类型.
// + 和 - 返回 0, 因为它们需要特殊处理(指针算术).
// > 和 >= 返回 0, 因为它们需要交换操作数.
static NodeKind get_binop(Token *tok) {
  if (equal(tok, "*"))  return ND_MUL;
  if (equal(tok, "/"))  return ND_DIV;
  if (equal(tok, "%"))  return ND_MOD;
  if (equal(tok, "<<")) return ND_SHL;
  if (equal(tok, ">>")) return ND_SHR;
  if (equal(tok, "==")) return ND_EQ;
  if (equal(tok, "!=")) return ND_NE;
  if (equal(tok, "<"))  return ND_LT;
  if (equal(tok, "<=")) return ND_LE;
  if (equal(tok, "&"))  return ND_BITAND;
  if (equal(tok, "^"))  return ND_BITXOR;
  if (equal(tok, "|"))  return ND_BITOR;
  if (equal(tok, "&&")) return ND_LOGAND;
  if (equal(tok, "||")) return ND_LOGOR;
  return 0;
}
```

### 3.3 核心 Pratt 函数

```c
// Pratt 表达式解析器.
//
// 这个函数替代了原来的 12 个二元运算符函数(logor 到 assign),
// 它们曾经构成了优先级调用链.
//
// pratt_expr 从左到右解析表达式, 利用绑定力(binding power)来
// 决定何时停止并返回. 核心思想:
//
//   - 左结合: 递归调用时传入 lbp 作为 min_bp
//     (当下一个运算符优先级相同或更低时停止)
//   - 右结合: 递归调用时传入 lbp - 1 作为 min_bp
//     (当下一个运算符优先级相同时继续)
//
// 本函数处理:
//   - 二元运算符: + - * / % & | ^ << >> == != < <= > >= && ||
//   - 赋值: = += -= *= /= %= &= |= ^= <<= >>=
//   - 三元: ?:(包括 GNU a ?: b 扩展)
//   - 逗号: ,
//   - 指针算术(+ 和 - 委托给 new_add/new_sub)
//   - > 和 >= 的操作数交换
//
// 本函数不处理:
//   - 类型转换表达式: 由 cast() 处理
//   - 一元前缀运算符: 由 unary() 处理
//   - 后缀运算符: 由 postfix() 处理
//   - 原子表达式(字面量, 标识符, sizeof 等): 由 primary() 处理
//
// 调用链: pratt_expr → cast → unary → postfix → primary
//        或 pratt_expr → cast → pratt_expr(右结合递归调用)
static Node *pratt_expr(Token **rest, Token *tok, int min_bp) {
  // 第一步: 解析左侧操作数(可以是类型转换, 一元, 后缀或原子表达式).
  Node *lhs = cast(&tok, tok);

  for (;;) {
    // --- 三元运算符(右结合, 特殊的三操作数语法) ---
    if (equal(tok, "?")) {
      int lbp = PREC_TERNARY;
      if (lbp < min_bp) {
        *rest = tok;
        return lhs;
      }
      Token *start = tok;
      tok = tok->next;

      // [GNU] 处理 `a ?: b`(省略中间操作数).
      // 编译为: tmp = a, tmp ? tmp : b
      if (equal(tok, ":")) {
        add_type(lhs);
        Obj *var = new_lvar("", lhs->ty);
        Node *lhs_assign = new_binary(ND_ASSIGN, new_var_node(var, start),
                                      lhs, start);
        Node *cond_node = new_node(ND_COND, start);
        cond_node->cond = new_var_node(var, start);
        cond_node->then = new_var_node(var, start);
        tok = tok->next;
        cond_node->els = pratt_expr(&tok, tok, lbp - 1);
        lhs = new_binary(ND_COMMA, lhs_assign, cond_node, start);
        continue;
      }

      // 标准三元: a ? b : c
      Node *node = new_node(ND_COND, start);
      node->cond = lhs;
      node->then = expr(&tok, tok);        // 中间表达式(允许逗号)
      tok = skip(tok, ":");
      node->els = pratt_expr(&tok, tok, lbp - 1);  // 右结合
      lhs = node;
      continue;
    }

    // --- 赋值运算符(右结合) ---
    if (is_assign_op(tok)) {
      int lbp = PREC_ASSIGN;
      if (lbp < min_bp) {
        *rest = tok;
        return lhs;
      }
      Token *start = tok;
      NodeKind op = get_assign_op(tok);
      tok = tok->next;

      // 右结合: 用 lbp - 1 解析右侧, 使得
      // a = b = c 被解析为 a = (b = c).
      Node *rhs = pratt_expr(&tok, tok, lbp - 1);

      if (op == 0) {
        // 普通赋值 "="
        lhs = new_binary(ND_ASSIGN, lhs, rhs, start);
      } else {
        // 复合赋值 "+=" "-=" 等.
        // to_assign() 处理 bitfield 和 atomic 的情况.
        lhs = to_assign(new_binary(op, lhs, rhs, start));
      }
      continue;
    }

    // --- 二元运算符 ---
    int lbp = get_lbp(tok);
    if (lbp == 0 || lbp < min_bp) {
      *rest = tok;
      return lhs;
    }

    Token *start = tok;
    NodeKind op = get_binop(tok);
    tok = tok->next;

    // 左结合: 传入 lbp(而非 lbp - 1)作为 min_bp.
    // 这意味着当下一个运算符优先级相同时,
    // lbp < min_bp 为假, 我们停止----从而实现左结合.
    Node *rhs = pratt_expr(&tok, tok, lbp);

    // 不能直接映射到单个 ND_* 节点的运算符需要特殊处理:

    // 1. + 和 -:指针算术需要调用 new_add/new_sub
    if (equal(start, "+")) {
      lhs = new_add(lhs, rhs, start);
      continue;
    }
    if (equal(start, "-")) {
      lhs = new_sub(lhs, rhs, start);
      continue;
    }

    // 2. > 和 >=:没有 ND_GT/ND_GE 节点----交换操作数
    if (equal(start, ">")) {
      lhs = new_binary(ND_LT, rhs, lhs, start);
      continue;
    }
    if (equal(start, ">=")) {
      lhs = new_binary(ND_LE, rhs, lhs, start);
      continue;
    }

    // 3. 其他所有二元运算符: 直接映射
    lhs = new_binary(op, lhs, rhs, start);
  }
}
```

### 3.4 修改 expr() 和 const_expr()

```c
// 改写前(递归下降):
//   expr = assign ("," expr)?
//
// 改写后(Pratt):
//   expr 只是调用 pratt_expr, 最小绑定力为逗号的优先级.
//   逗号运算符在 pratt_expr 内部以 PREC_COMMA 优先级处理.
//
// 现在是一行代码.
static Node *expr(Token **rest, Token *tok) {
  return pratt_expr(rest, tok, PREC_COMMA);
}

// const_expr 也是一行代码.
// 它解析常量表达式(用于 case 标签, 数组大小等),
// 并在编译期求值.
int64_t const_expr(Token **rest, Token *tok) {
  Node *node = pratt_expr(rest, tok, PREC_COMMA);
  return eval(node);
}
```

### 3.5 更新前向声明

删除原来 12 个函数的前向声明, 替换为:

```c
static Node *pratt_expr(Token **rest, Token *tok, int min_bp);
```

删除的前向声明:
```c
// 删除以下行:
static Node *assign(Token **rest, Token *tok);
static Node *logor(Token **rest, Token *tok);
static Node *logand(Token **rest, Token *tok);
static Node *bitor(Token **rest, Token *tok);
static Node *bitxor(Token **rest, Token *tok);
static Node *bitand(Token **rest, Token *tok);
static Node *equality(Token **rest, Token *tok);
static Node *relational(Token **rest, Token *tok);
static Node *shift(Token **rest, Token *tok);
static Node *add(Token **rest, Token *tok);
static Node *mul(Token **rest, Token *tok);
static Node *conditional(Token **rest, Token *tok);
```

添加的前向声明:
```c
static Node *pratt_expr(Token **rest, Token *tok, int min_bp);
```

## 4. 需要修改的外部调用点

以下函数原来调用 `assign()` 来解析表达式, 现在改为调用 `pratt_expr`:

### 4.1 funcall()(第 2892 行)

```c
// 改写前:
Node *arg = assign(&tok, tok);

// 改写后:
Node *arg = pratt_expr(&tok, tok, PREC_COMMA);
```

### 4.2 generic_selection()(第 2936,2952,2960 行)

```c
// 改写前(3 处):
Node *ctrl = assign(&tok, tok);
Node *node = assign(&tok, tok);

// 改写后:
Node *ctrl = pratt_expr(&tok, tok, PREC_COMMA);
Node *node = pratt_expr(&tok, tok, PREC_COMMA);
```

### 4.3 skip_excess_element()(第 917 行)

```c
// 改写前:
assign(&tok, tok);

// 改写后:
pratt_expr(&tok, tok, PREC_COMMA);
```

### 4.4 initializer2()(第 1253,1277 行)

```c
// 改写前:
Node *expr = assign(rest, tok);
// ...
init->expr = assign(rest, tok);

// 改写后:
Node *expr = pratt_expr(rest, tok, PREC_COMMA);
// ...
init->expr = pratt_expr(rest, tok, PREC_COMMA);
```

## 5. 不需要修改的部分

以下函数保持不变:

| 函数 | 原因 |
|------|------|
| `cast()` | 前缀类型转换, 不是二元运算符 |
| `unary()` | 前缀运算符, 不是二元运算符 |
| `postfix()` | 后缀运算符, 不是二元运算符 |
| `primary()` | 原子表达式 |
| `new_add()` / `new_sub()` | 指针算术辅助函数, 被 Pratt 循环调用 |
| `to_assign()` | 赋值转换辅助函数, 被 Pratt 循环调用 |
| `eval()` / `eval2()` / `eval_double()` | 编译期求值, 与解析无关 |
| `is_const_expr()` | 编译期常量检查 |
| `stmt()` / `compound_stmt()` | 语句解析 |
| `declspec()` / `declarator()` | 声明解析 |
| `initializer2()` / `lvar_initializer()` / `gvar_initializer()` | 初始化器解析 |

## 6. AST 输出对比

Pratt 改写产生**完全相同的 AST**. 以下是几个例子:

### 例 1:`a + b * c`(优先级)
```
ND_ADD(a, ND_MUL(b, c))
```

### 例 2:`a + b - c`(左结合)
```
ND_SUB(ND_ADD(a, b), c)
```

### 例 3:`a = b = c`(右结合)
```
ND_ASSIGN(a, ND_ASSIGN(b, c))
```

### 例 4:`a ? b : c`(三元)
```
ND_COND(a, b, c)
```

### 例 5:`a + b > c && d`(混合优先级)
```
ND_LOGAND(
  ND_LT(ND_ADD(a, b), c),  // 注意: > 被转换为 <,操作数交换
  d
)
```

### 例 6:`a, b, c`(逗号)
```
ND_COMMA(ND_COMMA(a, b), c)
```

### 例 7:`a += b * c`(复合赋值)
```
ND_COMMA(
  ND_ASSIGN(tmp, &a),           // tmp = &a
  ND_ASSIGN(*tmp, *tmp + b * c) // *tmp = *tmp + b * c
)
```
(由 `to_assign()` 转换)

## 7. 实现顺序

1. 在 `parse.c` 中添加优先级枚举, 查表函数(`get_lbp`,`get_binop`,`is_assign_op`,`get_assign_op`)
2. 添加 `pratt_expr` 函数
3. 修改 `expr()` 和 `const_expr()` 调用 `pratt_expr`
4. 修改所有外部调用点(`funcall`,`generic_selection`,`skip_excess_element`,`initializer2`)把 `assign()` 改为 `pratt_expr()`
5. 删除 12 个旧函数及其前向声明
6. 编译测试

## 8. 验证

```sh
make clean && make          # 编译通过
make test                   # stage 1 测试全部通过
make test-all               # stage 2 自举测试通过
```

特别关注的测试文件:
- `test/arith.c` -- 算术运算符优先级和结合性
- `test/control.c` -- 三元运算符
- `test/variable.c` -- 赋值运算符
- `test/macro.c` -- 预处理器(确保没有意外影响)
- `test/pointer.c` -- 指针算术

## 9. 代码量变化

| 项目 | 行数 |
|------|------|
| 删除: 12 个旧函数 | -320 行 |
| 删除: 12 个前向声明 | -12 行 |
| 新增: pratt_expr 函数 | +120 行 |
| 新增: 优先级表和辅助函数 | +60 行 |
| 修改: expr/const_expr | -10 行(变短了) |
| 修改: 外部调用点 | ~6 行(关键词替换) |
| **净减少** | **约 -160 行** |
