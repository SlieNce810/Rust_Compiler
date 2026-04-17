# Hopping 最小子集规范（V1）

本文档固化 `.hopping` 语言的最小可用子集，作为后续编译器和端侧解释器的统一输入规范。

## 1. 命名与文件扩展名

- 语言名称：`Hopping`
- 源文件扩展名：`.hopping`

兼容说明：

- 当前编译器仍可读取 `.mc`，但新文档和新示例统一使用 `.hopping`。

## 2. 语法最小子集

### 2.1 数据类型

- `int`
- `bool`

### 2.2 程序结构

- 顶层由函数组成
- 当前推荐入口：`func int main() { ... }`

### 2.3 语句

- 变量声明
- 赋值
- `if / else`
- `while`
- `return`

### 2.4 表达式

- 算术：`+ - * /`
- 比较：`< > ==`
- 字面量：整数、布尔值

## 3. BNF（最小语法）

```bnf
<program> ::= <function>*

<function> ::= "func" <type> <id> "(" <params>? ")" <block>
<params> ::= <param> ("," <param>)*
<param> ::= <type> <id>

<block> ::= "{" <statement>* "}"
<statement> ::=
    <declare_stmt>
  | <assign_stmt>
  | <if_stmt>
  | <while_stmt>
  | <return_stmt>

<declare_stmt> ::= <type> <id> ";"
<assign_stmt> ::= <id> "=" <expr> ";"
<if_stmt> ::= "if" "(" <expr> ")" <block> ("else" <block>)?
<while_stmt> ::= "while" "(" <expr> ")" <block>
<return_stmt> ::= "return" <expr> ";"

<expr> ::= <cmp_expr>
<cmp_expr> ::= <add_expr> (("<" | ">" | "==") <add_expr>)*
<add_expr> ::= <mul_expr> (("+" | "-") <mul_expr>)*
<mul_expr> ::= <factor> (("*" | "/") <factor>)*
<factor> ::= <id> | <number> | <bool> | "(" <expr> ")"

<id> ::= [a-zA-Z_][a-zA-Z0-9_]*
<number> ::= [0-9]+
<bool> ::= "true" | "false"
<type> ::= "int" | "bool"
```

## 4. 语义最小规则

### 4.1 变量与作用域

- 变量必须先声明再使用
- 块级作用域生效，子块可读父块变量
- 同一作用域内变量名不能重复声明

### 4.2 类型规则

- 赋值左右类型必须一致
- `if/while` 条件必须是 `bool`
- `return` 表达式类型必须和函数返回类型一致

### 4.3 运算规则

- `+ - * /` 只允许 `int/int`
- `< >` 只允许 `int/int` 比较，结果是 `bool`
- `==` 要求左右同类型，结果是 `bool`

### 4.4 编译失败条件（最小集）

- 未声明变量
- 重复声明
- 类型不匹配
- 非布尔条件
- 返回类型错误
- 非法字符或非法 token

## 5. 示例（点灯控制逻辑）

```hopping
func int main() {
    bool is_running;
    int led_state;
    int delay_counter;
    int delay_limit;
    int gpio_mode_reg;
    int gpio_output_reg;

    is_running = true;
    led_state = 0;
    delay_limit = 5000;
    gpio_mode_reg = 1;

    while (is_running) {
        if (led_state == 0) {
            led_state = 1;
        } else {
            led_state = 0;
        }

        gpio_output_reg = led_state;

        delay_counter = delay_limit;
        while (delay_counter > 0) {
            delay_counter = delay_counter - 1;
        }
    }

    return 0;
}
```

## 6. 与后续安全子集的关系

本规范是 V1 最小集合，目标是先稳定“可编译、可解释、可下发”。

后续会在此基础上增加：

- `float` 及更完整数值体系
- 资源类型（`gpio/uart/timer/...`）
- 状态检查规则（先绑定、再配置、后使用）
- 更强的编译期错误检查
