# chibicc 开发路线图

面向有编译器经验的开发者, 将 316 个 commit 合并为 15 个大任务.

---

## 任务 1: 基础表达式编译器
**commits: 5 个 · 产出: tokenize.c, parse.c, codegen.c 骨架**

| 子步骤 | commit | 实现内容 |
|--------|--------|----------|
| 1.1 | `0522e2d` | 从 `argv[1]` 读取数字字面量, 生成返回该值的 x86-64 可执行文件(`main` 函数返回该值). 建立项目结构: `main.c` 解析命令行, 调用 `as` 和 `ld`. |
| 1.2 | `bf7081f` | 在 tokenizer 中识别 `+` `-` 运算符, parser 支持左结合二元表达式, codegen 生成 `add`/`sub` 指令. |
| 1.3 | `a1ab0ff` | tokenizer 正式化: 将输入字符串拆分为 token 链表(`Token` 结构体), 支持空格分隔. |
| 1.4 | `84cfcaf` | 添加 `*` `/` 和 `()` 括号, 需要引入优先级(通过递归下降自然实现). |
| 1.5 | `bf9ab52` | 一元 `+`/`-` 运算符, parser 中 `unary()` 函数处理前缀. |

**关键实现细节**:
- 栈式代码生成: 每个表达式结果放 `%rax`, 二元运算 `push`/`pop`
- 不需要 IR, 直接从 AST 到 x86-64 汇编
- 错误信息: `error_at()` 在源码位置下方打印 `^` 指示符

**代码规模**: 约 200-300 行 C 代码

---

## 任务 2: 语句与控制流
**commits: 12 个 · 补充: parse.c 中 stmt/expr 函数**

| 子步骤 | commit | 实现内容 |
|--------|--------|----------|
| 2.1 | `76cae0a` | 多语句支持: `compount_stmt()` 解析 `stmt; stmt; ...` |
| 2.2 | `18ac283` | `{ }` 块语句, 引入 scope 概念(局部变量栈) |
| 2.3 | `72b8415` | `if` 语句: codegen 生成标签和条件跳转 |
| 2.4 | `f5d480f` | `for(init; cond; inc) body`:codegen 生成循环标签 |
| 2.5 | `1f3eb34` | `while(cond) body`:等价于 `for(;cond;)` |
| 2.6 | `6116cae` | `goto` 和 label 语句: 维护 `gotos` 和 `labels` 链表, pass2 解析前向 goto |
| 2.7 | `a4be55b` | label 和 typedef 名称冲突解析(C spec 要求) |
| 2.8 | `b3047f2` | `break`:跳出 switch/for/while/do, 通过 `brk_label` 链 |
| 2.9 | `3c83dfd` | `continue`:跳到循环 inc 部分, 通过 `cont_label` 链 |
| 2.10 | `044d9ae` | `switch/case`:case 用跳转表或 if-chain, codegen 收集 case 值生成 `cmp`+`je` |
| 2.11 | `447ee09` | `?:` 三元运算符: codegen 生成条件跳转, 结果在 `%rax` |
| 2.12 | `79f5de2` | 常量表达式: `eval()` 递归求值, 用于 `case`,`_Alignof`,数组大小等 |

**关键实现细节**:
- `break`/`continue` 通过嵌套链表实现: 每个循环/switch 设置 `brk_label`/`cont_label`,内层覆盖外层
- `switch` 的 codegen: 简单 case 用 if-chain, 连续 case 可优化为跳转表(但 chibicc 用 if-chain)
- goto 标签解析分两遍: 先收集所有 label 和 goto, 再绑定

**代码规模**: 约 300-400 行新增到 parse.c 和 codegen.c

---

## 任务 3: 变量与类型系统基础
**commits: 35 个 · 核心: parse.c declspec/declarator, type.c, codegen.c 变量存取**

| 子步骤 | commit | 实现内容 |
|--------|--------|----------|
| 3.1 | `1f9f3ad`→`482c26b` | 局部变量(单字母→多字母), 通过 `rbp-offset` 寻址 |
| 3.2 | `6cc1c1f` | `return` 语句: 结果已在 `%rax`,codegen 生成 epilogue |
| 3.3 | `863e2b8`→`a6bc4ab` | 指针 `&`/`*` 运算符 + 指针算术(`+`/`-` 自动乘以 sizeof) |
| 3.4 | `b4e82cf` | `int` 关键字, 变量定义必须声明类型 |
| 3.5 | `be38d63`→`4cedda2` | `char` 类型 + 字符串字面量(存入 `.data` 段) |
| 3.6 | `ad7749f`→`c2cc1d3` | 转义序列: `\n \t \a` 等 + `\xHH` 十六进制 + `\OOO` 八进制 |
| 3.7 | `5831eda` | int 从 8 字节改为 4 字节(x86-64 ABI) |
| 3.8 | `43c2f08`→`9d48eef` | `long`(8 字节)和 `short`(2 字节) |
| 3.9 | `a817b23`→`287906a` | 嵌套声明解析(`int *p, *q`)+ 复杂声明(函数指针等) |
| 3.10 | `74e3acc`→`8c3503b` | 函数声明 + `void` 类型 |
| 3.11 | `f46370e`→`a6b82da` | `long long` 别名 + `typedef` |
| 3.12 | `cb81a37` | char/short/int 使用 32 位寄存器(`%edi`/`%esi` 等) |
| 3.13 | `cfc4fa9`→`8b430a6` | type cast + usual arithmetic conversion(`type.c: get_common_type`) |
| 3.14 | `9e211cb`→`fdc80bc` | 未定义函数报错 + 返回值/参数类型自动转换 |
| 3.15 | `44bba96`→`aa0accc`→`48ba265` | `_Bool` + 字符字面量 `'x'` + `enum` |
| 3.16 | `3f59ce7`→`34ab83b`→`aaf1045` | `signed`/`unsigned` + `U`/`L`/`LL` 后缀 |
| 3.17 | `8b8f3de`→`6880a39`→`7ba6fe8` | unsigned 类型在运算中的处理(比较用无符号) |
| 3.18 | `b773554`→`1fad259` | 忽略 `const`/`volatile`/`auto`/`register`/`restrict`/`_Noreturn` + 参数名省略 |

**关键实现细节**:
- 类型系统核心在 `type.c`:`Type` 结构体有 `kind`,`size`,`align`,`base`(指针/数组指向的类型)
- `declspec()` 解析声明说明符(`int`/`char`/`long`/`unsigned`/`static`/`extern` 等组合)
- `declarator()` 解析声明器(`*p[10]`,`(*fp)(int)` 等)
- `usual_arith_conv()` 在 `type.c:170`:二元运算时统一类型
- 32 位寄存器规则: char/short 读入内存时符号扩展到 32 位

**代码规模**: 约 800-1000 行新增

---

## 任务 4: 函数完整支持
**commits: 28 个 · 核心: parse.c funcall/function, codegen.c 函数调用约定**

| 子步骤 | commit | 实现内容 |
|--------|--------|----------|
| 4.1 | `30a3992`→`964b1d2` | 函数调用: 0 参数 → 最多 6 个整型参数(rdi/rsi/rdx/rcx/r8/r9) |
| 4.2 | `6cb4220`→`aacc0cf` | 函数定义: 0 参数 → 最多 6 个参数, 生成 prologue/epilogue |
| 4.3 | `0b76634` | 合并 `Function` 和 `Var` 为统一的 `Obj` 结构体 |
| 4.4 | `736232f` | 文件作用域函数(非 main) |
| 4.5 | `a4fea2b` | for 循环内定义局部变量(作用域正确) |
| 4.6 | `157356c`→`319772b`→`127056d` | 全局变量对齐 + static 局部变量 + compound literal |
| 4.7 | `30b3e21`→`eb85527` | 无返回值的 `return` + static 全局变量 |
| 4.8 | `ee252e6` | `do...while` 循环 |
| 4.9 | `6a0ed71` | 栈帧 16 字节对齐(x86-64 ABI 要求) |
| 4.10 | `dcd4579` | 返回 bool/char/short 时正确截断到 32 位 |
| 4.11 | `58fc861`→`754a24f`→`197689a` | variadic 函数调用 + `va_start` + 参数数量检查 |
| 4.12 | `b29f052`→`9021f7f` | 栈传参: 超过 6 个参数时压栈 |
| 4.13 | `5e0f8c4`→`d63b1f4`→`c72df1c`→`d7bad96` | struct 作为参数/返回值传递(通过内存拷贝) |
| 4.14 | `b6d3cd0`→`603de50` | variadic 函数支持 >6 参数 + `va_copy` |
| 4.15 | `d06a8ac`→`c5953ba`→`53e8103` | 函数指针 + 函数衰减为指针 + 函数指针的 usual conversion |
| 4.16 | `31087f8`→`e5f4ca9` | inline 函数处理: 未被引用的 static inline 不生成代码 |
| 4.17 | `6a2dc5a` | `__attribute__((format(printf,...)))` 编译期检查 printf 参数 |

**关键实现细节**:
- System V AMD64 ABI: 整型前 6 个参数用 rdi/rsi/rdx/rcx/r8/r9, 浮点用 xmm0-xmm7
- struct 传参/返回: 小 struct 直接通过寄存器, 大 struct 通过内存(caller 分配 ret_buffer)
- variadic 函数: `va_area` 存在栈上, `va_start` 读取已知参数后的寄存器/栈值
- 函数指针: 类型系统中 `TY_FUNC` 类型, 调用时解引用

**代码规模**: 约 600-800 行新增

---

## 任务 5: 数组与 sizeof
**commits: 7 个 · 补充: type.c array_of, parse.c postfix/primary**

| 子步骤 | commit | 实现内容 |
|--------|--------|----------|
| 5.1 | `8b6395d` | 一维数组: `int a[10]`,codegen 用 `rbp-offset` 寻址 |
| 5.2 | `3ce1b2d` | 多维数组: `int a[2][3]`,base 类型递归 |
| 5.3 | `648646b` | `[]` 下标运算符: `a[i]` 等价于 `*(a+i)` |
| 5.4 | `3e55caf` | `sizeof`:整数常量表达式, 返回 `ty->size` |
| 5.5 | `29ed294` | 不完全数组类型: `int a[]`(前向声明) |
| 5.6 | `7963221` | 函数参数中数组衰减为指针 |
| 5.7 | `61a1055` | 不完全 struct 类型(前向声明) |

**关键实现细节**:
- 数组到指针衰减在 `type.c: add_type()` 中 `ND_VAR` case 处理
- `sizeof` 对数组返回 `元素大小 * 元素个数`,对指针返回 8
- 不完全类型在 `ty->size < 0` 时标记, 解析完 initializer 后确定大小

**代码规模**: 约 150-200 行新增

---

## 任务 6: 结构体与联合体
**commits: 16 个 · 核心: parse.c struct_decl/union_decl, type.c Member**

| 子步骤 | commit | 实现内容 |
|--------|--------|----------|
| 6.1 | `f814033` | struct 定义: `struct { int x; }`,`Member` 链表存储成员 |
| 6.2 | `9443e4b` | struct 成员对齐: 根据成员类型计算 offset 和 struct 整体 align |
| 6.3 | `dfec115` | 局部变量对齐 |
| 6.4 | `e1e831e` | struct tag:`struct Foo { int x; }`,通过 tag 查找 |
| 6.5 | `f0a018a` | `->` 运算符: `p->x` 等价于 `(*p). x` |
| 6.6 | `11e3841` | union: 成员共享偏移量 0, size 取最大成员 |
| 6.7 | `bef0543` | struct 赋值: 逐字节拷贝 |
| 6.8 | `cc852fe`→`441a89b`→`54c2b3b` | bitfield:`int x:3`,codegen 用移位和掩码操作 |
| 6.9 | `17ea802`→`c302a96` | 零宽度 bitfield + 禁止对 bitfield 取地址 |
| 6.10 | `c3075b3` | 匿名 struct/union(C11) |
| 6.11 | `44bea4c` | `__attribute__((packed))`:成员 offset 连续不填充 |
| 6.12 | `b35d148` | `__attribute__((aligned(N)))`:设置 struct 整体对齐 |
| 6.13 | `90d1f7f` | `=` 和 `?:` 运算符支持 struct 成员访问 |

**关键实现细节**:
- struct layout 在 `parse.c: struct_decl()` 中计算: 遍历成员, 按对齐要求分配 offset
- bitfield 用 `bit_offset` 和 `bit_width` 表示, codegen 用 `shl`/`shr`/`and` 操作
- union 所有成员 offset=0, size=max(member sizes)
- struct 赋值在 codegen 中逐字节拷贝(未优化为 memcpy)

**代码规模**: 约 400-500 行新增

---

## 任务 7: 初始化器系统
**commits: 18 个 · 核心: parse.c initializer/lvar_initializer/gvar_initializer**

| 子步骤 | commit | 实现内容 |
|--------|--------|----------|
| 7.1 | `22dd560` | 局部变量初始化器: `int x = 5`,codegen 生成赋值序列 |
| 7.2 | `ae0a37d`→`a754732` | 数组多余元素零初始化 + 多余 initializer 元素跳过 |
| 7.3 | `0d71737`→`5b95533` | 字符串字面量初始化数组 + 省略数组长度 |
| 7.4 | `e9d2c46`→`aca19dd` | struct 局部初始化器 + struct 赋值初始化 |
| 7.5 | `483b194` | union 局部初始化器 |
| 7.6 | `bbfe3f4`→`eeb62b6`→`1eae5ae` | 全局变量初始化(scalar/string/struct/union): 写入 `.data` 段 |
| 7.7 | `efa0f33`→`a58958c`→`fde464c` | 省略括号 + 多余花括号 + 末尾逗号 |
| 7.8 | `3d216e3` | 未初始化全局变量写入 `.bss` 而非 `.data` |
| 7.9 | `824543b`→`cd688a8` | flexible array member + 初始化 |
| 7.10 | `c618c3b`→`835cd24`→`691c4fa` | designated initializer:`a[2]=5`,`{[0]=1}`,GNU 省略 `=` |
| 7.11 | `67f5834`→`31dc1df`→`95eb5b0` | struct/union designated initializer + 匿名成员 |

**关键实现细节**:
- `Initializer` 是树结构, 递归处理嵌套(`int a[2][3] = {{1,2,3},{4,5,6}}`)
- `InitDesg` 链表记录当前初始化位置(数组索引, struct 成员)
- 局部初始化器: codegen 生成一系列赋值指令
- 全局初始化器: 必须是常量表达式, 直接写入 `.data`/`.bss` 段
- designated initializer: 解析时跳到指定位置, 未指定的元素零初始化

**代码规模**: 约 600-800 行新增到 parse.c

---

## 任务 8: 浮点数
**commits: 11 个 · 核心: codegen.c 浮点指令, type.c 浮点类型**

| 子步骤 | commit | 实现内容 |
|--------|--------|----------|
| 8.1 | `1e57f72` | 浮点常量: tokenizer 解析 `3.14`,`1.0f` 等, 存入 `token.fval` |
| 8.2 | `29de46a` | float/double 变量和 cast:`float x = 3.14`,`movss`/`movsd` 指令 |
| 8.3 | `cf9ceec` | 浮点比较: `==` `!=` `<` `<=`,用 `ucomiss`/`ucomisd` 指令 |
| 8.4 | `83f76eb` | 浮点算术: `+` `-` `*` `/`,用 `addss`/`mulsd` 等指令 |
| 8.5 | `0ce1093` | 浮点在 if/while/do/`!`/`?:`/`||`/`&&` 中的处理 |
| 8.6 | `8ec1ebf`→`c6b3056` | 浮点参数/返回值传递: 用 xmm0-xmm7 寄存器 |
| 8.7 | `8b14859` | 默认参数提升: float → double(variadic 函数) |
| 8.8 | `e452cf7` | variadic 函数中浮点参数: va_start 时保存浮点寄存器 |
| 8.9 | `ffea421` | 浮点常量表达式: `eval_double()` 递归求值 |
| 8.10 | `9bf9612` | `long double`:x87 80 位扩展精度, 用 `fld`/`fstp` 指令 |

**关键实现细节**:
- 浮点用 XMM 寄存器(xmm0-xmm7), 整型用 GP 寄存器
- 浮点返回值在 `%xmm0`,整型返回值在 `%rax`
- `long double` 特殊处理: 16 字节对齐, 用 x87 FPU 指令
- 常量表达式求值: `eval_double()` 处理所有浮点运算

**代码规模**: 约 300-400 行新增

---

## 任务 9: 预处理器
**commits: 43 个 · 独立文件: preprocess.c**

| 子步骤 | commit | 实现内容 |
|--------|--------|----------|
| 9.1 | `1e1ea39`→`146c7b3` | 空预处理器 + null 指令 `#` |
| 9.2 | `d367510`→`ec149f6`→`d138864` | `#include "..."` + `-E` 选项 |
| 9.3 | `bf6ff92`→`aa570f3`→`c6e81d2`→`e7a1857` | `#if`/`#endif`/`#else`/`#elif`:条件编译 |
| 9.4 | `97d33ad`→`9ad60e4` | objlike `#define` + `#undef` |
| 9.5 | `2651448`→`acce002` | `#if` 中宏展开 + 防止重复展开(hideset) |
| 9.6 | `1f80f58` | `#ifdef`/`#ifndef` |
| 9.7 | `dec3b3f`→`b9ad3e4`→`dd4306c`→`c7d7ce0`→`1313fc6` | funclike macro:0/多参数 + 空参数 + 括号参数 + 防重复展开 |
| 9.8 | `8f6f792`→`8f561ae` | `#` stringize + `##` token paste |
| 9.9 | `769b5a0` | 所有测试用 chibicc 自己的预处理器编译 |
| 9.10 | `5cb2f89`→`a8d76ad` | `defined()` + 非宏标识符在 `#if` 中替换为 0 |
| 9.11 | `8075582`→`b33fe0e` | 宏展开保留换行和空格 + 行继续 `\` |
| 9.12 | `d85fc4f`→`a1dd621`→`a939a7a` | `#include <...>` + `-I` + 默认 include 路径 |
| 9.13 | `e7fdc2e`→`5f5a850` | `#error` + 预定义宏 `__STDC__` 等 |
| 9.14 | `6f17071`→`dc01f94`→`ba6b4b6`→`82ba010` | `__FILE__`/`__LINE__`/`__VA_ARGS__`/`__func__`/`__FUNCTION__` |
| 9.15 | `ab4f1e1`→`7746e4e` | 相邻字符串字面量拼接 + 宽字符字面量识别 |
| 9.16 | `7cbfd11`→`5322ea8` | stdarg.h/stdbool.h 等 stub 头文件 + `va_arg` |
| 9.17 | `12a9e75` | chibicc 能编译自身(自举成功) |
| 9.18 | `3f2c2d5`→`fc69f5c`→`be8b6f6` | pp-number token + `-D`/`-U` |
| 9.19 | `c61c0d0`→`aaf20fb` | `#line` + GNU line marker directive |
| 9.20 | `3381448`→`083c275`→`007e526` | `__VA_OPT__` + `,##__VA_ARGS__` + GCC 风格 variadic macro |
| 9.21 | `e27417f`→`0e77f3d` | `__DATE__`/`__TIME__`/`__COUNTER__` |

**关键实现细节**:
- 预处理器是 token 流到 token 流的转换, 不构建 AST
- hideset 机制防止宏递归展开: 每个 token 记录已展开过的宏名集合
- `#if` 求值: 递归展开后替换非宏标识符为 0, 然后 `eval` 常量表达式
- `defined()` 在 `#if` 中特殊处理: 不展开括号内的宏名

**代码规模**: preprocess.c 约 1000 行, stub 头文件若干

---

## 任务 10: 编译器驱动与链接
**commits: 35+ 个 · 核心: main.c**

| 子步骤 | commit | 实现内容 |
|--------|--------|----------|
| 10.1 | `725badf`→`d9ea597` | 拆分 main.c 为多文件 + 从文件读取源码 |
| 10.2 | `a0388ba`→`6c0a429` | `-o`/`--help` + 行注释和块注释 |
| 10.3 | `ca8b243`→`cd832a3` | block scope + 测试从 shell 脚本改写为 C |
| 10.4 | `6647ad9`→`1c91d19` | token 行号预计算 + `.file`/`.loc` 调试指令 |
| 10.5 | `f3d9613` | 拆分 cc1(内部编译阶段)和 driver(选项解析+子进程调用) |
| 10.6 | `140b433`→`8b726b5` | 调用 `as` 汇编 + 调用 `ld` 链接 |
| 10.7 | `b833cd0` | 支持多文件输入 |
| 10.8 | `2bdc6b8`→`b1fdddf`→`2c91da5` | 内存缓冲输出 + 忽略 GCC 选项 + `-Wall` |
| 10.9 | `5257ee0`→`9c36dd7` | 16 字节对齐数组 + `main` 隐式返回 0 |
| 10.10 | `85e46b1`→`6d344ed` | tentative definition + `-fcommon`/`-fno-common` |
| 10.11 | `b377284` | thread-local 变量 |
| 10.12 | `8f5ff07`→`ee0a951`→`4064871` | `-include` + `-x` + `-E implies -xc` |
| 10.13 | `bc25279`→`c32f0e2`→`8d130ab`→`d56dd2f` | `-l`/`-s` + 符号大小信息 + `.a`/`.so` 识别 |
| 10.14 | `e0bf168` | long double(x87 80 位) |
| 10.15 | `86785fc`→`c0f0614`→`d48d9e5`→`a6c6622`→`f10bceb` | `-fpic`/`-fPIC` + include 路径缓存 + include guard 优化 + `#pragma once` + `#include_next` |
| 10.16 | `1e9b6dd`→`4e5de36`→`c8df787`→`d1bc9a4`→`469f159` | `-static`/`-shared`/`-L`/`-Wl,`/`-Xlinker` |

**关键实现细节**:
- driver 模式(`main()`)解析命令行, fork+exec 调用 cc1/as/ld 子进程
- cc1 模式(`-cc1`)执行实际编译: tokenize → preprocess → parse → codegen
- 链接器调用需要找到 `crt1.o`/`crti.o`/`crtbegin.o` 等 CRT 文件
- `-fpic` 影响全局变量寻址方式(GOTPCREL)和 TLS 访问

**代码规模**: main.c 约 650 行, 加上各处小改动

---

## 任务 11: 自举 (Stage 2)
**commits: 3 个 · Makefile: stage2 目标**

| 子步骤 | commit | 实现内容 |
|--------|--------|----------|
| 11.1 | `5d15431` | Makefile 添加 `stage2/` 目标: 用 stage1 chibicc 编译自身 |
| 11.2 | `769b5a0` | 所有测试用 chibicc 自己的预处理器编译(验证预处理器自举) |
| 11.3 | `12a9e75` | 完整自举: chibicc 能编译自身, stage2 chibicc 能运行测试 |

**关键实现细节**:
- Stage 2 编译流程: `./chibicc -c -o stage2/main.o main.c` → 链接成 `stage2/chibicc`
- `make test-all` 先跑 stage1 测试, 再跑 stage2 测试
- 自举验证的是: 编译器自身代码(约 5000 行 C)能被正确编译

**代码规模**: Makefile 约 30 行新增

---

## 任务 12: Unicode 与宽字符串
**commits: 17 个 · 核心: tokenize.c/unicode.c 字面量解析**

| 子步骤 | commit | 实现内容 |
|--------|--------|----------|
| 12.1 | `74bcec5` | 规范化换行符(`\r\n` → `\n`) |
| 12.2 | `c31886a` | `\u`/`\U` Unicode 转义序列 |
| 12.3 | `a57c661`→`454618c`→`2dac3af` | 宽字符字面量 `L'x'`/`u'x'`/`U'x'` |
| 12.4 | `57b21fe`→`9cabe1f`→`c467ee6`→`cae061a` | UTF-8/16/32/宽字符串字面量 |
| 12.5 | `36230e0`→`6adba75` | UTF-16/32 字符串初始化器 |
| 12.6 | `e4491b8` | 定义 `__STDC_UTF_16__`/`__STDC_UTF_32__` |
| 12.7 | `0e5d250` | 标识符允许多字节 UTF-8 字符 |
| 12.8 | `adb8b98` | GNU 扩展: `$` 作为标识符字符 |
| 12.9 | `2382777` | 普通字符串与 L/u/U 字符串拼接 |
| 12.10 | `2b2fa25` | 跳过 UTF-8 BOM 标记 |

**关键实现细节**:
- `unicode.c` 实现 UTF-8 编解码(`encode_utf8`/`decode_utf8`)和 `display_width`
- 宽字符串在 tokenizer 中解析为字节序列, 存入 `token.str`
- 不同编码的字符串拼接需要统一转换

**代码规模**: unicode.c 约 170 行, tokenizer 约 200 行新增

---

## 任务 13: 高级语言特性
**commits: 12 个 · 散布在 parse.c/codegen.c/type.c**

| 子步骤 | commit | 实现内容 |
|--------|--------|----------|
| 13.1 | `9dae234` | statement expression (GNU):`({ stmt; expr; })` |
| 13.2 | `4f165ec`→`f0c98e0` | labels-as-values (GNU):`&&label` + `goto *ptr` |
| 13.3 | `d90c73b`→`3d5550e` | case ranges (GNU):`case 1 ... 5:` + array range designator |
| 13.4 | `7d80a51` | `typeof`:获取表达式的类型 |
| 13.5 | `1433b40` | `__builtin_types_compatible_p` |
| 13.6 | `1faab48` | `_Generic`:C11 泛型选择 |
| 13.7 | `aee7891`→`e28a612` | sizeof(function type) (GNU) + `?:` 省略中间操作数 (GNU) |
| 13.8 | `a253516` | 基本 `asm` 语句 |
| 13.9 | `77275c5` | `alloca()`:运行时栈分配 |
| 13.10 | `e8667af`→`07f9010`→`2fa8f48` | VLA: sizeof + 指针算术 + sizeof(typename) |
| 13.11 | `1b99bad` | `offsetof` |
| 13.12 | `7a1f816` | `void` 作为参数列表(`f(void)` 等价于 `f()`) |

**关键实现细节**:
- statement expression: codegen 生成 block, 返回最后一个 expr_stmt 的值
- labels-as-values:`&&label` 生成 label 地址, `goto *ptr` 间接跳转
- VLA: 运行时在栈上分配(`alloca`), 类型中有 `vla_len` 和 `vla_size`
- `_Generic`:匹配类型列表, 选择匹配的表达式

**代码规模**: 约 300-400 行散布在多个文件

---

## 任务 14: 原子操作与线程局部存储
**commits: 5 个 · 核心: codegen.c 原子指令, parse.c atomic 解析**

| 子步骤 | commit | 实现内容 |
|--------|--------|----------|
| 14.1 | `ca4755` | `atomic_compare_exchange`:`lock cmpxchg` 指令 |
| 14.2 | `80ea9d4` | `atomic_exchange`:`xchg` 指令 |
| 14.3 | `d69a11d` | `_Atomic` 类型 + atomic `++`/`--`/`op=` 运算符 |
| 14.4 | `0a5d08c` | stdatomic.h 完整 stub |
| 14.5 | `b377284` | thread-local 变量: `__thread`/`_Thread_local`,codegen 用 `%fs` 段寄存器 |

**关键实现细节**:
- atomic 操作在 codegen 中用 x86 lock 前缀指令
- TLS 变量: non-PIC 用 `mov %fs: offset`,PIC 用 `__tls_get_addr@PLT`
- `_Atomic` 在类型系统中标记, 影响 codegen 生成原子指令

**代码规模**: 约 150-200 行新增

---

## 任务 15: 构建优化与工具链集成
**commits: 20+ 个 · 核心: hashmap.c, preprocess.c include guard, main.c -M 选项**

| 子步骤 | commit | 实现内容 |
|--------|--------|----------|
| 15.1 | `0aad326`→`30520e5`→`655954e`→`f694413` | hashmap 实现 + 宏名/block scope/keyword 查找改用 hashmap |
| 15.2 | `d0c4667`→`95d5a46`→`57c1d4e`→`db850f3`→`fb5cfe5`→`7aa72e4`→`c3edffb` | `-M` 系列: 自动生成 makefile 依赖 |
| 15.3 | `c0f0614`→`d48d9e5` | include 路径搜索缓存 + include guard 优化(跳过已 include 的头文件) |
| 15.4 | `a6c6622`→`f10bceb` | `#pragma once` + `#include_next` |
| 15.5 | `fb49370`→`2ed3fda` | 第三方程序测试脚本(cpython.sh 等) |
| 15.6 | `44bea4c`→`b35d148` | `__attribute__((packed))`/`__attribute__((aligned))` |

**关键实现细节**:
- hashmap 用开放寻址法, 支持按 key 或 key+len 查找
- `-M` 选项: 预处理阶段收集 `#include` 的文件列表, 输出 makefile 格式
- include guard 优化: `#ifndef FOO / #define FOO` 模式检测到后, 跳过整个头文件
- 第三方测试: 编译 cpython, libpng 等真实项目验证编译器正确性

**代码规模**: hashmap.c 约 130 行, 其他散布

---

## 总体代码规模

| 文件 | 行数 | 说明 |
|------|------|------|
| parse.c | 2832 | 最大文件, 递归下降 parser + initializer |
| codegen.c | 1422 | x86-64 代码生成 |
| preprocess.c | 1022 | C 预处理器 |
| main.c | 654 | 编译器驱动 |
| tokenize.c | 692 | 词法分析 |
| type.c | 307 | 类型系统 |
| chibicc.h | 389 | 公共头文件 |
| unicode.c | 170 | UTF-8 处理 |
| hashmap.c | 134 | 哈希表 |
| strings.c | 26 | 字符串工具 |
| **总计** | **~8500** | |

**测试文件**: 41 个 `.c` 文件, 共约 2500 行

---

## 建议实现顺序

有编译器经验的话, 推荐按以下顺序推进:

1. **任务 3** (类型系统基础) → 2. **任务 4** (函数) → 3. **任务 5** (数组) → 4. **任务 6** (struct/union) → 5. **任务 7** (初始化器) → 6. **任务 2** (控制流) → 7. **任务 8** (浮点) → 8. **任务 9** (预处理器) → 9. **任务 10** (驱动与链接)

先搭类型系统和函数框架, 再补控制流和浮点, 最后做预处理器和驱动. 每个大任务内部可以快速推进, 遇到边界 case 再参考对应 commit.
